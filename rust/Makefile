serve: 
	./maelstrom/maelstrom serve

b:
	cargo build    

r: b
	./target/debug/gossip-glommers

m: b 
	./maelstrom/maelstrom test -w unique-ids --bin ./target/debug/gossip-glommers --node-count 3 --time-limit 30 --rate 100  --log-stderr  --availability total --nemesis partition

mp: b 
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/gossip-glommers --node-count 10 --time-limit 500  --log-stderr

