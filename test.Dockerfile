# Test ARG instructions with and without defaults
ARG VERSION=1.0.0
ARG BUILD_DATE

# Test ENV instructions
ENV APP_VERSION=$VERSION
ENV BUILD_TIMESTAMP=$BUILD_DATE
ENV DEBUG=true

# Test ADD with URL download
ADD https://raw.githubusercontent.com/rust-lang/rust/master/README.md ./README.md

# Test single-line RUN command
RUN echo "Building version $APP_VERSION"

# Test multi-line RUN command
RUN echo "Starting multi-line command" && \
    echo "Build timestamp: $BUILD_TIMESTAMP" && \
    echo "Debug mode: $DEBUG" && \
    echo "End of multi-line command"

# Test RUN with file interaction
RUN cat README.md | head -n 5 && \
    echo "=== File contents above ===" 