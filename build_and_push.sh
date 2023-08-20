# Usage: bash build_and_push.sh X.Y.Z
docker build --tag dewpoint --platform=linux/amd64 .

docker tag dewpoint:latest dewpoint:$1
docker tag dewpoint:$1 assensam/dewpoint:$1
docker push assensam/dewpoint:$1
docker push assensam/dewpoint:latest
