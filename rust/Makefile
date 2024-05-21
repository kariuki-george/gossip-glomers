serve: 
	./maelstrom/maelstrom serve

echo-build:
	cargo build  --bin echo  

echo: echo-build
	 ./target/debug/echo

m-echo: echo-build 
	./maelstrom/maelstrom test -w echo --bin ./target/debug/echo --node-count 2 --time-limit 10  --log-stderr


