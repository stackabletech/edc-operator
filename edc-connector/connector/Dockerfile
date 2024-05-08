# -buster is required to have apt available
FROM openjdk:22-slim-buster

# by default curl is not available, so install it
RUN apt update && apt install curl -y

WORKDIR /app


# Copy config
COPY ./resources /resources
# Copy jar
COPY ./build/libs/dataspace-connector.jar /app

ENV EDC_FS_CONFIG = $EDC_FS_CONFIG

ENTRYPOINT java -jar dataspace-connector.jar
