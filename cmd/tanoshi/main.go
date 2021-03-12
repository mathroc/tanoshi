package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"path"

	rice "github.com/GeertJohan/go.rice"
	"github.com/faldez/tanoshi/internal/config"
	"github.com/faldez/tanoshi/internal/database"
	"github.com/faldez/tanoshi/internal/history"
	"github.com/faldez/tanoshi/internal/library"
	"github.com/faldez/tanoshi/internal/proxy"
	"github.com/faldez/tanoshi/internal/server"
	"github.com/faldez/tanoshi/internal/source"
	"github.com/faldez/tanoshi/internal/update"
)

var (
	srv server.Server
	cfg config.Config
)

func main() {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		log.Fatalf("Can't get home directory, please provide path to config file: %s", err.Error())
	}
	var configPath *string = flag.String("config", path.Join(homeDir, ".config/tanoshi/config.yml"), "path to config file")

	flag.Parse()

	cfg, err = config.Load(*configPath)
	if err != nil {
		log.Fatalf("Error load config: %s", err.Error())
	}

	if cfg.ExtensionURL != "" && cfg.ExtensionURL != source.RepositoryURL {
		source.RepositoryURL = cfg.ExtensionURL
	}

	db, err := database.Open(cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("Error connect database: %s", err.Error())
	}

	db.AutoMigrate(&source.Source{}, &source.Manga{}, &source.Chapter{}, &source.Page{})

	sourceRepo := source.NewRepository(db)
	libraryRepo := library.NewRepository(db)
	historyRepo := history.NewRepository(db)
	updateRepo := update.NewRepository(db)

	sourceHandler, err := source.NewHandler(sourceRepo, cfg.LocalDir)
	if err != nil {
		log.Fatalln(err.Error())
	}

	libraryHandler := library.NewHandler(libraryRepo)
	historyHandler := history.NewHandler(historyRepo)
	updateHandler := update.NewHandler(updateRepo)

	proxy := proxy.NewProxy()

	configHandler := config.NewHandler(&cfg)

	conf := rice.Config{
		LocateOrder: []rice.LocateMethod{rice.LocateEmbedded, rice.LocateAppended, rice.LocateFS},
	}
	box, err := conf.FindBox("../../web/build")
	if err != nil {
		log.Fatalf("error opening rice.Box: %s\n", err)
	}

	srv = server.NewServer(sourceHandler, libraryHandler, historyHandler, updateHandler, proxy, configHandler, box)
	srv.RegisterHandler()

	err = srv.Run(fmt.Sprintf(":%s", cfg.Port))
	log.Fatalln(err)
}
