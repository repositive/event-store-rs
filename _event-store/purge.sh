#!/bin/sh

curl -XDELETE -u guest:guest http://localhost:15673/api/queues/%2f/save_and_aggregate_receive-_eventstore.EventReplayRequested/contents
curl -XDELETE -u guest:guest http://localhost:15673/api/queues/%2f/save_and_aggregate_send-_eventstore.EventReplayRequested/contents
curl -XDELETE -u guest:guest http://localhost:15673/api/queues/%2f/save_and_aggregate_receive-some_namespace.TestEvent/contents
