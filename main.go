package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"strconv"
)

func main() {
	checkerFlag := flag.Bool("checker", false, "run the checker")
	flag.Parse()

	c := make(chan struct{})
	setTimer(checkArgs(*checkerFlag), c)

	resources, processes, goal, err := parseFile("./" + flag.Arg(0))
	if err != nil {
		fmt.Printf("Error while parsing: %v\n", err)
		fmt.Println("Exiting...")
		os.Exit(1)
	}

	// `ubik` is the the name of a resource that is produced by every process.
	finite, ubik := buildGraph(resources, processes, goal)
	currentResources := make(map[string]int)
	for i := range resources {
		currentResources[resources[i].name] = resources[i].quantity
	}

	if *checkerFlag {
		checker(currentResources, processes)
	}

	end, t := schedule(currentResources, processes, finite, c, ubik)

	s := buildOutput(currentResources, processes, end, finite, t)
	writeOutput(s)

	// // Uncomment to print the schedule to the terminal.
	// fmt.Print(s)
}

func checkArgs(ch bool) float64 {
	// Default waiting time: 1 second
	usage := "usage: ./stock <file> [<waiting_time>] | ./stock -checker <file> <log_file>"
	f := 1.0
	if flag.NArg() > 2 {
		log.Println("too many arguments")
		log.Fatal(usage)
	}
	if flag.NArg() < 1 {
		log.Println("not enough arguments")
		log.Fatal(usage)
	}
	if flag.NArg() == 2 {
		if ch {
			return f
		}
		g, err := strconv.ParseFloat(flag.Arg(1), 64)
		if err != nil {
			fmt.Println(fmt.Errorf("error parsing wait time: %v", err))
			fmt.Println("Did you mean to run the checker?")
			fmt.Println(usage)
			os.Exit(1)
		}
		f = g
	}
	return f
}
