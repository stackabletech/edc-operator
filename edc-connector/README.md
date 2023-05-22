# Stackable EDC build

Based on [IONOS EDC](https://github.com/Digital-Ecosystems/edc-ionos-s3)

This is our build of the EDC, including the IONOS extension.

The Connector repository stated

> **It is expected that everyone who wants to use the EDC will create their own launcher, customized
to the implemented use cases.**

[up until the 19th of May 2023.](https://github.com/eclipse-edc/Connector/commit/edb35971c00b9c1007018c5325b0cc5fc5b2b1ba#diff-b335630551682c19a781afebcf4d07bf978fb1f8ac04c6bf87428ed5106870f5)

This is intended as a stable source for the jar, as neither the EDC Project nor IONOS provide versioned jars. The jar should also work well together with our Dockerfile. In the future there should be official jars available (most likely) and then this custom build will not be necessary anymore.


## Build

    ./gradlew clean build

The build will create a `dataspace-connector.jar` in `./connector/build/libs`.

It is uploaded [here](https://repo.stackable.tech/#browse/browse:packages:edc%2Fdataspace-connector.jar).

It is currently not versioned, as this is still in an experimental state. As stated above, there should be official, versioned releases in the future.
