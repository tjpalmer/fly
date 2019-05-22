package main

import (
	"fmt"
	"log"
	"net/http"
)

func main() {
	fmt.Printf("Hello, world!\n")
	http.HandleFunc("/", handler)
	log.Fatal(http.ListenAndServe(":8080", nil))
}

func handler(w http.ResponseWriter, r *http.Request) {
	fmt.Fprintf(w, "Hi from %s", r.URL.Path[1:])
}
