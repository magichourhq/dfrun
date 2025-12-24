# Test comment handling
FROM ubuntu:22.04

# ARG with default
ARG VERSION=1.0.0
ARG BUILD_TYPE=release

# ARG without default (should use env or prompt)
ARG REPO_URL

# ENV with = syntax
ENV APP_VERSION=$VERSION
ENV BUILD_MODE=$BUILD_TYPE

# ENV with space syntax
ENV WORKSPACE /opt/app

# Multi-line RUN command
RUN echo "Starting build..." && \
    echo "Version: $APP_VERSION" && \
    echo "Mode: $BUILD_MODE" && \
    echo "Workspace: $WORKSPACE"

# Nested variable in command
RUN export FULL_VERSION="${APP_VERSION}-${BUILD_MODE}" && \
    echo "Full version: $FULL_VERSION"

# WORKDIR should be ignored but not break parsing
WORKDIR /app

# Test that commands still run in original directory
RUN pwd && ls -la

# ADD with URL
ADD https://raw.githubusercontent.com/magichourhq/dfrun/main/README.md ./

# Complex multi-line with comments in between (tricky!)
RUN echo "Step 1" && \
    echo "Step 2" && \
    echo "Step 3"

# ENV overwriting previous ENV
ENV APP_VERSION=${APP_VERSION}-final

# Use the overwritten variable
RUN echo "Final version: $APP_VERSION"

# Unsupported instructions (should be ignored gracefully)
COPY . /app
EXPOSE 8080
CMD ["echo", "done"]
LABEL maintainer="test"
USER nobody
VOLUME /data

# One more RUN to confirm we're still working
RUN echo "All done! APP_VERSION=$APP_VERSION BUILD_MODE=$BUILD_MODE"
