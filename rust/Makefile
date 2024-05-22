serve: 
	./maelstrom/maelstrom serve

b:
	cargo build    

r: b
	./target/debug/gossip-glommers

m: b 
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/gossip-glommers --node-count 2 --time-limit 10  --log-stderr

mp: b 
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/gossip-glommers --node-count 6 --time-limit 20  --log-stderr

