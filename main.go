package main

// I use torrents to move a lot of my media around
// the .torrent files are tiny but frequently clutter my /downloads folder
// to unclutter my downloads folder I wrote this quick utility that monitors the /downloads directory
// any time a .torrent file is downloaded, it gets automatically moved to a /downloads/torrents directory
import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/fsnotify/fsnotify" // the important library
)

func main() {
	watcher, err := fsnotify.NewWatcher() // watcher is the object that monitors the directory
	if err != nil {
		log.Fatal(err)
	}
	defer watcher.Close() //  close the watcher when we're done or it runs forever

	done := make(chan bool) // this is just a channel to keep the program running until closed

	go func() { // this goroutine listens FOR events from the watcher
		for {
			select {
			case event, ok := <-watcher.Events: // if there is an event, check if it's a file creation event
				if !ok {
					return
				}
				if event.Op&fsnotify.Create == fsnotify.Create { // if it is a file creation event, check if it's a .torrent file
					if strings.HasSuffix(event.Name, ".torrent") {
						moveFile(event.Name)
					}
				}
			case err, ok := <-watcher.Errors: // if there is an error, log it
				if !ok {
					return
				}
				log.Println("error:", err)
			}
		}
	}()

	downloadsDir := "C:\\Users\\RikGa\\Downloads" // this is the directory we want to monitor
	err = watcher.Add(downloadsDir)
	if err != nil {
		log.Fatal(err)
	}
	<-done // block forever
}

func moveFile(filePath string) {
	torrentsDir := "C:\\Users\\RikGa\\Downloads\\torrents" // this is the directory we want to move the .torrent files to
	if _, err := os.Stat(torrentsDir); os.IsNotExist(err) {
		os.Mkdir(torrentsDir, os.ModePerm)
	}

	fileName := filepath.Base(filePath) // get the file name from the path
	newLocation := filepath.Join(torrentsDir, fileName)

	err := os.Rename(filePath, newLocation)
	if err != nil {
		log.Println("error moving file:", err)
	} else {
		fmt.Printf("Moved %s to %s\n", filePath, newLocation)
	}
}
