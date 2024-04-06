apt-get update

apt-get isntall -y \
    curl \
    git \
    openssl \
    build-essential

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
