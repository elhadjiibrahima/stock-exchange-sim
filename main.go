package main

import (
	"fmt"
	"os"

	"stockexchange/production"
)

func main() {
	// Read the file
	// arg := os.Args
	// if len(arg) != 3 {
	// 	fmt.Println("Usage : go run . <File name> <waiting_time seconde>")
	// 	return
	// }
	// timer, err := strconv.ParseFloat(arg[2], 32)
	// if err != nil {
	// 	fmt.Println("Error while parsing `" + arg[2] + "`")
	// 	os.Exit(0)
	//}
	data, err := os.ReadFile("examples/simple")
	if err != nil {
		fmt.Println("File reading error", err)
		return

	}
	fmt.Println(string(data))

	production := production.Production{}
	production.Timeout = true
	fmt.Println(production.Timeout)
}
