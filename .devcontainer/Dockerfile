ARG VARIANT=bullseye
FROM debian:${VARIANT}
ENV DEBIAN_FRONTEND=noninteractive
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

# Arguments
ARG CONTAINER_USER=esp
ARG CONTAINER_GROUP=esp
ARG TOOLCHAIN_VERSION=1.67.0.0
ARG ESP_IDF_VERSION=release/v4.4
ARG ESP_BOARD=esp32s3
ARG INSTALL_RUST_TOOLCHAIN=install-rust-toolchain.sh
ARG NODE_VERSION=v18.15.0

# Install dependencies
RUN apt-get update \
    && apt-get install -y git curl gcc clang ninja-build libudev-dev unzip xz-utils\
    python3 python3-pip python3-venv libusb-1.0-0 libssl-dev pkg-config libtinfo5 libpython2.7 \
    && apt-get clean -y && rm -rf /var/lib/apt/lists/* /tmp/library-scripts


# Adding sources for yarn in root mode
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - 
RUN echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list 
RUN apt-get update \
    && apt-get install -y yarn && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/* /tmp/library-scripts

# Set users
RUN adduser --disabled-password --gecos "" ${CONTAINER_USER}
RUN adduser ${CONTAINER_USER} sudo
RUN echo "${CONTAINER_USER} ALL=(ALL:ALL) NOPASSWD:ALL" >> /etc/sudoers 
USER ${CONTAINER_USER}
WORKDIR /home/${CONTAINER_USER}

# Install Rust toolchain, extra crates and esp-idf
ENV PATH=${PATH}:/home/${CONTAINER_USER}/.cargo/bin:/home/${CONTAINER_USER}/opt/bin

ADD --chown=${CONTAINER_USER}:${CONTAINER_GROUP} \
    https://github.com/esp-rs/rust-build/releases/download/v${TOOLCHAIN_VERSION}/${INSTALL_RUST_TOOLCHAIN} \
    /home/${CONTAINER_USER}/${INSTALL_RUST_TOOLCHAIN}

RUN chmod a+x ${INSTALL_RUST_TOOLCHAIN} \
    && ./${INSTALL_RUST_TOOLCHAIN} \
    --extra-crates "ldproxy cargo-espflash wokwi-server web-flash" \
    --export-file /home/${CONTAINER_USER}/export-esp.sh \
    --esp-idf-version "${ESP_IDF_VERSION}" \
    --minified-esp-idf "YES" \
    --build-target "${ESP_BOARD}" \
    && rustup component add clippy rustfmt

# Install nvm and node
ENV NVM_DIR /home/${CONTAINER_USER}/.nvm
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash \
    && . $NVM_DIR/nvm.sh \
    && nvm install ${NODE_VERSION} \
    && nvm alias default ${NODE_VERSION} \
    && nvm use default

# Emscripten SDK to compile c into wasm
RUN cd /home/${CONTAINER_USER} \
    && git clone https://github.com/emscripten-core/emsdk.git \
    && chown -R ${CONTAINER_USER}:${CONTAINER_GROUP} emsdk \
    && cd emsdk \
    && ./emsdk install latest \
    && ./emsdk activate latest \
    && . ./emsdk_env.sh

# Activate Emscripten environment
RUN echo "source /home/${CONTAINER_USER}/emsdk/emsdk_env.sh" >> ~/.bashrc

# Enable nvm
RUN echo "source /home/${CONTAINER_USER}/.nvm/nvm.sh" >> ~/.bashrc

# Activate ESP environment
RUN echo "source /home/${CONTAINER_USER}/export-esp.sh" >> ~/.bashrc

# Upgrade .bashrc
RUN echo '' >> ~/.bashrc \
    && echo '# s to show git status' >> ~/.bashrc \
    && echo 'alias s="git status -u"' >> ~/.bashrc \
    && echo '# ports to show open ports' >> ~/.bashrc \
    && echo 'alias ports="sudo lsof -i -P -n | grep LISTEN"' >> ~/.bashrc 

# write version of node, nvm, yarn, rustc, cargo and emcc into version.md
RUN echo "Node: $(node -v)" > version.md \
    && echo "NPM: $(nvm -v)" >> version.md \
    && echo "Yarn: $(yarn -v)" >> version.md \
    && echo "Rustc: $(rustc -V)" >> version.md \
    && echo "Cargo: $(cargo -V)" >> version.md \
    && echo "Emscripten: $(emcc -v)" >> version.md

CMD [ "/bin/bash" ]
