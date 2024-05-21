serve: 
	./maelstrom/maelstrom serve

echo-build:
	go build  -o bin/echo ./cmd/echo/main.go

echo: echo-build
	 ./bin/echo

m-echo: echo-build 
	./maelstrom/maelstrom test -w echo --bin ./bin/echo --node-count 1 --time-limit 10  --log-stderr


