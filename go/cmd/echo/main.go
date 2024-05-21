package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
)

type Message struct {
	Source string `json:"src"`
	Dest   string `json:"dest"`
	Body   Body   `json:"body"`
}

type Body struct {
	Type      string `json:"type"`
	MsgId     uint64 `json:"msg_id"`
	InReplyTo uint64 `json:"in_reply_to"`

	Node_id string   `json:"node_id"`
	Nodes   []string `json:"node_ids"`
}

type Node struct {
	message_counter uint64
	Id              string
	Neighbours      []string
}

var node = Node{}

func main() {

	scanner := bufio.NewScanner(os.Stdin)

	for scanner.Scan() {
		text := scanner.Text()

		msg := Message{}
		json.Unmarshal([]byte(text), &msg)

		switch msg.Body.Type {
		case "echo":
			handleWrite(handleEcho(msg))
		case "echo_ok":
			continue
		case "init":
			handleWrite(handleInit(msg))
		}

		node.message_counter += 1

	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Error:", err)
	}
}

func handleWrite(msg Message) {
	output := bufio.NewWriter(os.Stdout)
	data, _ := json.Marshal(msg)

	output.Write(data)
	output.WriteString("\n")
	output.Flush()

}

func handleInit(msg Message) Message {
	node.Id = msg.Body.Node_id
	node.Neighbours = msg.Body.Nodes

	return Message{
		Source: node.Id,
		Dest:   msg.Source,
		Body: Body{
			Type:      "init_ok",
			InReplyTo: msg.Body.MsgId,
			MsgId:     node.message_counter,
		},
	}

}

func handleEcho(msg Message) Message {
	return Message{
		Source: msg.Dest,
		Dest:   msg.Source,
		Body: Body{
			Type:      "echo_ok",
			MsgId:     node.message_counter,
			InReplyTo: msg.Body.MsgId,
		},
	}

}
