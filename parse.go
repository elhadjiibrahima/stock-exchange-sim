package main

import (
	"bufio"
	"fmt"
	"os"
	"regexp"
	"strconv"
	"strings"
)

func parseFile(config string) ([]resource, []process, goal, error) {
	file, err := os.Open(config)
	if err != nil {
		return nil, nil, goal{}, err
	}
	defer file.Close()

	var resources []resource
	var processes []process
	var g goal
	foundGoal := false

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := string(scanner.Text())

		// Ignore comments and empty lines.
		if strings.HasPrefix(line, "#") || len(strings.TrimSpace(line)) == 0 {
			continue
		}

		// Parse resources.
		resource, err := parseResource(line)
		if err == nil {
			resources = append(resources, resource)
			continue
		}

		// Parse goal.
		g, err = parseGoal(line)
		if err == nil {
			if foundGoal {
				return nil, nil, goal{}, fmt.Errorf("multiple goals found")
			}
			foundGoal = true
			continue
		}

		// Parse processe.
		process, err := parseProcess(line)
		if err == nil {
			processes = append(processes, process)
			continue
		}

		// If the line is not a comment, resource, goal, or process, return an error.
		return nil, nil, goal{}, fmt.Errorf("invalid line: `%s`", line)
	}

	// Missing resources, processes, or goal.
	if len(resources) == 0 {
		return nil, nil, goal{}, fmt.Errorf("no resources found")
	}
	if len(processes) == 0 {
		return nil, nil, goal{}, fmt.Errorf("no processes found")
	}
	if !foundGoal {
		return nil, nil, goal{}, fmt.Errorf("no goal found")
	}

	// Other errors.
	if err := scanner.Err(); err != nil && err.Error() != "EOF" {
		return nil, nil, goal{}, err
	}

	return resources, processes, g, nil
}

func parseResource(s string) (resource, error) {
	pattern := `([^:;()]+):(\d+)`
	if !regexp.MustCompile(pattern).MatchString(s) {
		return resource{}, fmt.Errorf("invalid resource: %s", s)
	}
	sl := strings.Split(s, ":")
	name := sl[0]
	quantity, err := strconv.Atoi(sl[1])
	if err != nil {
		return resource{}, fmt.Errorf("invalid resource: %s", s)
	}
	return resource{name, quantity, true}, nil
}

func parseGoal(s string) (goal, error) {
	pattern := `^(optimize:\()(time;)?([^:;()]+)\)$`
	if !regexp.MustCompile(pattern).MatchString(s) {
		return goal{}, fmt.Errorf("invalid goal: %s", s)
	}
	var product string
	time := false
	if strings.Contains(s, "time") {
		time = true
		semicolonIndex := strings.Index(s, ";")
		closingParenthesisIndex := strings.Index(s, ")")
		if semicolonIndex != -1 && closingParenthesisIndex != -1 && closingParenthesisIndex > semicolonIndex {
			product = s[semicolonIndex+1 : closingParenthesisIndex]
		}
	} else {
		closingParenthesisIndex := strings.Index(s, ")")
		if closingParenthesisIndex != -1 {
			product = s[10:closingParenthesisIndex]
		}
	}
	return goal{product, time}, nil
}

func parseProcess(s string) (process, error) {
	resourcePattern := `([^:]+):(\d+)`
	listOfResourcesPattern := `(` + resourcePattern + `;)*(` + resourcePattern + `)`
	pattern := `^([^:]+):\(` + listOfResourcesPattern + `\):\(` + listOfResourcesPattern + `\):(\d+)$`
	if !regexp.MustCompile(pattern).MatchString(s) {
		return process{}, fmt.Errorf("invalid process: %s", s)
	}

	parts, err := splitProcess(s)
	if err != nil {
		return process{}, err
	}

	name := parts[0]

	ingredientsStringSlice := strings.Split(parts[1], ";")
	ingredients := make([]resource, len(ingredientsStringSlice))
	for i, v := range ingredientsStringSlice {
		ingredient, err := parseResource(v)
		if err != nil {
			return process{}, fmt.Errorf("invalid process: %s", s)
		}
		ingredients[i] = ingredient
	}

	productsStringSlice := strings.Split(parts[2], ";")
	products := make([]resource, len(productsStringSlice))
	for i, v := range productsStringSlice {
		product, err := parseResource(v)
		if err != nil {
			return process{}, fmt.Errorf("invalid process: %s", s)
		}
		products[i] = product
	}

	time, err := strconv.Atoi(parts[3])
	if err != nil {
		return process{}, fmt.Errorf("invalid process: %s", s)
	}

	return process{name: name,
		ingredients: ingredients,
		products:    products,
		time:        time}, nil
}

func splitProcess(line string) ([]string, error) {
	parts := strings.SplitN(line, ":(", 2)
	if len(parts) < 2 {
		return nil, fmt.Errorf("invalid line: %s", line)
	}

	firstPart := parts[0]

	parts = strings.SplitN(parts[1], "):(", 2)
	if len(parts) < 2 {
		return nil, fmt.Errorf("invalid line: %s", line)
	}

	secondPart := parts[0]

	parts = strings.SplitN(parts[1], "):", 2)
	if len(parts) < 2 {
		return nil, fmt.Errorf("invalid line: %s", line)
	}

	thirdPart := parts[0]
	fourthPart := parts[1]

	return []string{firstPart, secondPart, thirdPart, fourthPart}, nil
}
