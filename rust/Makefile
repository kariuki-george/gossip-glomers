serve: 
	./maelstrom/maelstrom serve

b:
	cargo build    

r: b
	./target/debug/gossip-glommers

br:
	cargo build --release	

m: b 
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/gossip-glommers --node-count 3 --time-limit 30 --rate 100  --log-stderr  

mp: br 
	./maelstrom/maelstrom test -w broadcast --bin ./target/release/gossip-glommers --node-count 4 --time-limit 30 --rate 50  --log-stderr

