#!/bin/bash
#
# jq 1.6 parses all numbers (including bignums) into IEEE-754 doubles
# and then potentially writes them out in scientific notation, which cannot be parsed by SERDE/Substrate.
# So we ask Perl to reformat the floats as bignums if possible.
#
# Of course this is a hack, but it works for the numbers currently in our chain specs.
#
exec perl -0777 -MJSON::PP -E '$s=<>; $j=JSON::PP->new->ascii->pretty->allow_nonref->allow_bignum;$p=$j->decode($s);say $j->encode($p)'
