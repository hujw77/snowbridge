import { ApiPromise, WsProvider } from "@polkadot/api"
import { BlockHash, Hash, EventRecord } from "@polkadot/types/interfaces"
import Redis from "ioredis"
import async from "async"
import { Vec } from "@polkadot/types"

const NETWORK = "staging"
const PARA_ID = 1000

function* range(m: number, n: number) {
    for (var i = m; i < n; i++) {
        yield i
    }
}

let sleep = (ms: number) => {
    return new Promise((resolve) => setTimeout(resolve, ms))
}

const redis = new Redis({
    host: "127.0.0.1",
    port: 6379,
})

let processEventRecords = (api: ApiPromise, records: Vec<EventRecord>): Map<number, Hash> => {
    let paraHeads: Map<number, Hash> = new Map()
    for (let { event } of records) {
        if (event.section != "paraInclusion" || event.method != "CandidateIncluded") {
            continue
        }
        let receipt = api.createType("CandidateReceipt", event.data[0])
        paraHeads.set(receipt.descriptor.paraId.toNumber(), receipt.descriptor.paraHead)
    }
    return paraHeads
}

let key = (network: string, paraHead: Hash) => `paraHead:${network}:${paraHead.toHex()}`

interface Task {
    api: ApiPromise
    paraId: number
    blockNumber: number
}

let worker: async.AsyncWorker<Task> = async ({ api, paraId, blockNumber }) => {
    console.log("Processing block ", blockNumber)
    let blockHash = await api.rpc.chain.getBlockHash(blockNumber)
    const records = await api.query.system.events.at(blockHash)
    let paraHeads = processEventRecords(api, records)
    for (let [id, head] of paraHeads) {
        if (id != paraId) {
            continue
        }
        redis.set(key(NETWORK, head), blockHash.toHex())
    }
}

let main = async () => {
    // Construct
    let wsProvider = new WsProvider("wss://kusama-rpc.polkadot.io")
    let api = await ApiPromise.create({
        provider: wsProvider,
        types: {
            SpecVersion: "u32",
        },
    })

    // new events

    let startSubscribeBlock: Hash = null

    let unsub = await api.query.system.events(async (events) => {
        console.log(`Processing block ${events.createdAtHash.toHex()}`)
        if (startSubscribeBlock === null) {
            startSubscribeBlock = events.createdAtHash
        }
        let paraHeads = processEventRecords(api, events)
        for (let [id, head] of paraHeads) {
            if (id != PARA_ID) {
                continue
            }
            await redis.set(key(NETWORK, head), events.createdAtHash.toHex())
        }
    })

    while (true) {
        if (startSubscribeBlock !== null) {
            break
        }
        await sleep(6000)
    }

    // historical events
    let queue = async.queue(worker, 24)
    let endHeader = await api.rpc.chain.getHeader(startSubscribeBlock)
    let endBlockNumber = endHeader.number.toNumber()

    queue.error(function (err, task) {
        console.error("Task experienced an error: ", err)
    })

    for (let i of range(endBlockNumber - 100, endBlockNumber)) {
        queue.push({
            api,
            paraId: PARA_ID,
            blockNumber: i,
        })
    }

    await queue.drain()

    // redis.disconnect()

    // await api.disconnect()
}

main().catch((error) => console.log(error))
