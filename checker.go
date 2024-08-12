package main

import (
	"bufio"
	"flag"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
)

func checker(resources map[string]int, processes []process) {
	file, err := os.Open("./" + flag.Arg(1))
	if err != nil {
		log.Fatalf("error parsing log file: %v", err)
	}
	defer file.Close()

	p := make(map[string]*process)
	for i := range processes {
		p[processes[i].name] = &processes[i]
	}

	scanner := bufio.NewScanner(file)
	isFirstLine := true
	for scanner.Scan() {
		if isFirstLine {
			isFirstLine = false
			continue
		}

		line := scanner.Text()
		ln := strings.Split(line, ":")
		if len(ln) != 2 {
			break
		}
		cycle := strings.TrimSpace(ln[0])
		name := ln[1]
		fmt.Println("Evaluating:", line)

		if _, err := strconv.Atoi(cycle); err != nil {
			fmt.Printf("Error while parsing: %s\n", err)
			fmt.Println("Exiting...")
			os.Exit(1)
		}

		process, ok := p[name]
		if !ok {
			fmt.Println("Error detected")
			fmt.Printf("process %s not found\n", name)
			fmt.Println("Exiting...")
			os.Exit(1)
		}

		for _, ingredient := range process.ingredients {
			if resources[ingredient.name] < ingredient.quantity {
				fmt.Println("Error detected")
				fmt.Printf("at %s stock insufficient\n", line)
				fmt.Println("Exiting...")
				os.Exit(1)
			}
			resources[ingredient.name] -= ingredient.quantity
		}
		for _, product := range process.products {
			resources[product.name] += product.quantity
		}
	}
	fmt.Println("Trace completed, no error detected.")
	os.Exit(0)
}
