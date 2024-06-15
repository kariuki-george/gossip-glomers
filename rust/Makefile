serve: 
	./maelstrom/maelstrom serve

b:
	cargo build    

r: b
	./target/debug/gossip-glommers

br:
	cargo build --release	

m: b 
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/gossip-glommers  --node-count 5 --time-limit 20 --rate 10   --nemesis partition

mp: br 
	./maelstrom/maelstrom test -w broadcast --bin ./target/release/gossip-glommers --node-count 4 --time-limit 30 --rate 50  --log-stderr

