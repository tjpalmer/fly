package cmd

import (
	"fmt"
	"log"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use: "fly",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("Hi!")
	},
}

// Execute program.
func Execute() {
	if err := rootCmd.Execute(); err != nil {
		log.Fatal(err)
	}
}
