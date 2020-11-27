#!/bin/zsh

# Zulip requests are capped at 5000

# USAGE
#
# Go to your Zulip account and get an API key: https://zulip.com/api/api-keys
# insert your email / key below, for API_EMAIL and API_KEY
# run the script
# NOTE: This is a zsh script, and may need a little retooling to work with other .sh-es.
# NOTE: This script will pull *all* message data with the "has":"link" narrow. It's quite a lot, so be wary of using it if you're streaming video or doing anything else in the next ~10 minutes.
# NOTE: If you want pretty-printed versions of all this, you'll need python installed.


API_EMAIL=YOUR_API_EMAIL_HERE
API_KEY=YOUR_API_KEY_HERE
SITE=https://recurse.zulipchat.com

if [ ! -f "./streams.json" ]
then
    echo "no cached streams found.  cURLing streams.json"
    curl -sSX GET -G ${SITE}/api/v1/streams \
	 -u ${API_EMAIL}:${API_KEY} \
	 > streams.json
fi

STREAMS=$(jq -r '.streams [] .name' streams.json)

ANCHOR_NUMBER=2436412
BEFORE=1
AFTER=4500


STRMS=$(echo $STREAMS | xargs -0)
set -f
IFS='
'
foreach STREAM (${=STRMS})
echo "---------------"
# STREAM=$(echo $STREAM | tr -d\")
MSGID=0
MSGMARK=15
echo "from messageID: $MSGID \n msmrk: $MSGMARK"
while [ $MSGID -ne $MSGMARK ]
do
    echo "curling stream: $STREAM"
    echo "curling from id: $MSGID"
    SAVESTRM=$(echo $STREAM | tr -d '[:blank:]')
    curl -sSX GET -G ${SITE}/api/v1/messages \
	 -u $API_EMAIL:$API_KEY \
	 -d "anchor=$MSGID" \
	 -d "num_before=$BEFORE" \
	 -d "num_after=$AFTER" \
	 --data-urlencode narrow='[{"operand" : "link", "operator" : "has"}]' \
	 --data-urlencode narrow="[{\"operand\" : \"$STREAM\", \"operator\" : \"stream\"}]" > "$SAVESTRM.$MSGID.json"
    echo "should have saved to $STREAM.$MSGID.json"
    echo "curled $MSGID"
    MSGMARK=$MSGID
    MSGID=$(jq '.messages [-1] .id' "$SAVESTRM.$MSGID.json")
done
end

mkdir pretty

foreach file (./*)
cat $file | python -m json.tool > "pretty/$file.p"
end

