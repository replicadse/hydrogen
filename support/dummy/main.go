package main

import (
	"net/http"
	"time"

	"github.com/gorilla/mux"
)

func handler(w http.ResponseWriter, req *http.Request) {
	// just return success
	w.WriteHeader(200)
}

func main() {
	r := mux.NewRouter()
	r.HandleFunc("/", handler).
		Methods("POST").
		Schemes("http")

	srv := &http.Server{
		Addr: "0.0.0.0:8080",
		// No Slowloris
		WriteTimeout: time.Second * 15,
		ReadTimeout:  time.Second * 15,
		IdleTimeout:  time.Second * 60,
		Handler:      r,
	}
	srv.ListenAndServe()
}
