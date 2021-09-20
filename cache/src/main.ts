import { ApiPromise, WsProvider } from "@polkadot/api"
import { BlockHash, Hash, EventRecord } from "@polkadot/types/interfaces"
import Redis from "ioredis"
import async from "async"
import { Vec } from "@polkadot/types"
import yargs from "yargs"
import winston from "winston"

interface Task {
    blockNumber: number
}

interface Context {
    log: winston.Logger
    api: ApiPromise
    redis: Redis.Redis
    keyExpiry: number
    paraId: number,
}


// Expire keys after 3 weeks
const DEFAULT_EXPIRY = 1814400

function* range(m: number, n: number) {
    for (var i = m; i < n; i++) {
        yield i
    }
}

let sleep = (ms: number) => {
    return new Promise((resolve) => setTimeout(resolve, ms))
}

let processEventRecords = (api: ApiPromise, paraId: number, records: Vec<EventRecord>): Hash[] => {
    let heads: Hash[] = []
    for (let { event } of records) {
        if (event.section != "paraInclusion" || event.method != "CandidateIncluded") {
            continue
        }
        let receipt = api.createType("CandidateReceipt", event.data[0])
        if (receipt.descriptor.paraId.toNumber() == paraId) {
            heads.push(receipt.descriptor.paraHead)
        }
    }
    return heads
}

let key = (paraHead: Hash) => `para-head:${paraHead.toHex()}`

let makeWorker = (ctx: Context): async.AsyncWorker<Task> => {
    let { log, api } = ctx
    return async ({ blockNumber }) => {
        log.info("Processing block %d", blockNumber)
        let blockHash = await api.rpc.chain.getBlockHash(blockNumber)
        const records = await api.query.system.events.at(blockHash)
        let paraHeads = processEventRecords(api, records)
        await writeKeys(ctx, blockHash, paraHeads)
    }
}

let writeKeys = async ({ log, redis, keyExpiry }: Context, blockHash: BlockHash, paraHeads: Hash[]) => {
    for (let head of paraHeads) {
        try {
            await redis.set(key(head), blockHash.toHex(), "EX", keyExpiry)
        } catch (error) {
            log.error("Failed to write key to redis", error)
        }
    }
}

let subscribeNewEvents = async (ctx: Context): Promise<number> => {
    // first block received from subscription
    let startBlock: Hash = null

    let { log, api, redis, keyExpiry, paraId } = ctx
    let unsub = await api.query.system.events(async (events) => {
        log.info("Processing block %s", events.createdAtHash.toHex())
        if (startBlock === null) {
            startBlock = events.createdAtHash
        }
        let allParaHeads = processEventRecords(api, events)
        if (allParaHeads.has(paraId)) {
            redis.set(
                key(allParaHeads.get(paraId)),
                events.createdAtHash.toHex(),
                "EX", keyExpiry
            )
        }
    })

    while (true) {
        if (startBlock !== null) {
            break
        }
        await sleep(6000)
    }
    let header = await api.rpc.chain.getHeader(startBlock)
    return header.number.toNumber()
}

let queryHistoricalEvents = async (ctx: Context, startBlock: number, endBlock: number) => {
    // Query historical events in range [checkpoint, startSubscribeBlock)
    let queue = async.queue(makeWorker(ctx), 16)

    queue.error(function (err, task) {
        ctx.log.error("Task failed: %s", err)
    })

    for (let i of range(startBlock, endBlock)) {
        queue.push({
            blockNumber: i,
        })
    }

    await queue.drain()
}


let main = async () => {
    const argv = yargs.options({
        "polkadot-url": { type: "string", demandOption: true },
        "redis-host": { type: "string", demandOption: true },
        "redis-port": { type: "number", demandOption: true },
        "para-id": { type: "number", demandOption: true },
        "expiry": { type: "number", default: DEFAULT_EXPIRY },
      }).argv as any;

    const log = winston.createLogger({
        level: "debug",
        format: winston.format.combine(
            winston.format.timestamp(),
            winston.format.splat(),
            winston.format.json(),
        ),
        transports: [
            new winston.transports.Console(),
        ],
        exitOnError: false
    });

    let wsProvider = new WsProvider(argv.polkadotUrl)
    let api = await ApiPromise.create({
        provider: wsProvider,
        types: {
            SpecVersion: "u32",
        },
    })

    const redis = new Redis({
        host: argv.redisHost,
        port: argv.redisPort,
    })

    redis.on('error', error => {
        log.error(error)
        redis.quit()
       })

    let context: Context = {
        log,
        api,
        redis: redis,
        keyExpiry: argv.expiry,
        paraId: argv.paraId,
    }

    let subscribeStartBlock = await subscribeNewEvents(context)
    await queryHistoricalEvents(context, subscribeStartBlock - 50, subscribeStartBlock)
}

main().catch((error) => console.log(error))
