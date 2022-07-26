# Connection to connection chat example

This chat application allows connection to connection messages to be sent through hydrogen and a OpenFaas backend. The `deploy.sh` script builds the image, loads it into `KinD` (local kubernetes cluster) and deploys the function.

## Commands

* `!send $target_connection_id $message`
