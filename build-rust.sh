# For convenience only. Build a new image.
# As always, make this executable with chmod +x ./build-docker.sh
docker build -t ferrugem_rust -f Dockerfile.rust .

# tag with:
# docker tag [...] rtakahashimuralis/ferrugem_rust:[version]
# push with:
# docker push rtakahashimuralis/ferrugem_rust:[version]