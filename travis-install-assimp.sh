#!/bin/sh
set -ex
wget https://github.com/assimp/assimp/archive/v3.2.tar.gz
tar -xzvf v3.2.tar.gz
cd assimp-3.2 && cmake . && make && sudo make install && cd ..
