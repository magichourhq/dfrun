ARG VERSION=1.0.0
ENV APP_VERSION=$VERSION
ENV BUILD_TYPE=release
RUN echo "Building version $APP_VERSION in $BUILD_TYPE mode"
RUN echo "VERSION=$VERSION" > version.txt
RUN echo "APP_VERSION=$APP_VERSION" >> version.txt
RUN echo "BUILD_TYPE=$BUILD_TYPE" >> version.txt