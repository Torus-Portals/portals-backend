scratchpad

sudo scp portals-backend-dev.tar ubuntu@ec2-13-56-200-232.us-west-1.compute.amazonaws.com:~

scp -i ./broch-1.pem ./portals-backend-dev.tar ubuntu@ec2-13-56-200-232.us-west-1.compute.amazonaws.com:~



portals-dev-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com -p 5432 -U "postgres" -W


Building a docker image:
$ docker build --tag <image-name> . // if in the same directory as the Dockerfile



DATABASE_URL=postgres://brochstilley:Hellothere123!@localhost:5432/portals_main
PORTALS_MAIN_HOST=127.0.0.1:8088



docker run -d -it \
--publish 8088:8088 \
--publish 5432:5432 \
-e DATABASE_URL='postgres://postgres:qwerty123456!@portals-dev-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com:5432/portals_main'  \
-e PORTALS_MAIN_HOST=0.0.0.0:8088 \
--name pb1 \
portals-backend-dev:latest




ec2-54-183-208-228.us-west-1.compute.amazonaws.com

ssh -i ./broch-1.pem ec2-user@ec2-54-183-208-228.us-west-1.compute.amazonaws.com

scp -i ./broch-1.pem ./portals-backend-dev.tar ubuntu@ec2-13-56-200-232.us-west-1.compute.amazonaws.com:~


sudo docker run --rm -i -t portals-backend-no-cmd bash

docker run --rm -i -t \
--publish 8088:8088 \
--publish 5432:5432 \
-e DATABASE_URL='postgres://postgres:qwerty123456!@portals-local-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com:5432/portals_main'  \
-e PORTALS_MAIN_HOST=0.0.0.0:8088 \
--name pb1-no-cmd \
portals-backend-no-cmd:latest

postgres://postgres:qwerty123456!@portals-local-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com:5432/portals_main

psql -h portals-local-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com -p 5432 -U "postgres" -W "qwerty123456!"

// psql -h <host> -p <port> -u <database>
// psql -h <host> -p <port> -U <username> -W <password> <database>


sudo netstat -ntlp | grep LISTEN

psql -h portals-dev-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com -p 5432 -U "postgres" -W

export DATABASE_URL='postgres://postgres:qwerty123456!@portals-local-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com:5432/portals_main'



docker run --rm -i -t \
--publish 8088:8088 \
--publish 5432:5432 \
-e DATABASE_URL='postgres://postgres:qwerty123456!@portals-dev-1.cece4u7qvrsx.us-west-1.rds.amazonaws.com:5432/portals_main'  \
-e PORTALS_MAIN_HOST=0.0.0.0:8088 \
-e 
--name pb1-no-cmd \
portals-backend-no-cmd:latest