serve: 
	./maelstrom/maelstrom serve -p 3000

b:
	cargo build    

r: b
	./target/debug/gossip-glommers

br:
	cargo build --release	

m: b 
	./maelstrom/maelstrom test -w kafka --bin ./target/debug/gossip-glommers  --node-count 1 --concurrency 2n --time-limit 20 --rate 1000

mp: br 
	./maelstrom/maelstrom test -w broadcast --bin ./target/release/gossip-glommers --node-count 4 --time-limit 30 --rate 50  --log-stderr

