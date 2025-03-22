# waf/build.sh

go build -o waf-proxy.o
go mod tidy
echo "Executing ./waf-proxy.o"
./waf-proxy.o