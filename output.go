package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"path"
	"strings"
)

func buildOutput(resources map[string]int,
	processes []process,
	end int,
	finite bool,
	t string) string {
	var builder strings.Builder
	builder.WriteString("Processes scheduled:\n")
	var someScheduled bool
	if finite {
		for _, process := range processes {
			for i := 0; i <= end; i++ {
				if process.start == i {
					for j := 0; j < process.iterations; j++ {
						someScheduled = true
						str := fmt.Sprintf(" %d:%s\n", process.start, process.name)
						builder.WriteString(str)
					}
				}
			}
		}
	} else {
		builder.WriteString(t)
		if len(t) > 0 {
			someScheduled = true
		}
	}
	if !someScheduled {
		builder.WriteString(" none\n")
	}
	builder.WriteString(fmt.Sprintf("No more process doable at cycle %d.\n", end+1))
	builder.WriteString("Stock left:\n")
	for resource, quantity := range resources {
		builder.WriteString(fmt.Sprintf(" %s:%d\n", resource, quantity))
	}
	s := builder.String()
	return s
}

func writeOutput(s string) {
	filename := flag.Arg(0)
	basename := strings.TrimSuffix(filename, path.Ext(filename))
	logFilename := basename + ".log"
	file, err := os.Create("./" + logFilename)
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	_, err = io.WriteString(file, s)
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Output written to", logFilename)
}
