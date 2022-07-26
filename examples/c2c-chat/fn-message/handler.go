package function

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
)

// Handle a serverless request
func Handle(req []byte) string {
	request := map[string]string{}
	json.Unmarshal(req, &request)
	command := strings.SplitN(request["message"], " ", 3)
	if command[0] != "!send" {
		panic("not send command")
	}

	http.Post(fmt.Sprintf("http://hydrogen-hydrogen-gateway.hydrogen.svc.cluster.local:8080/connections/%v/_send",
		command[1]), "text/plain", strings.NewReader(command[2]))
	return ""
}
