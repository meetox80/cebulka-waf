// cebulka-waf/modules/registry.go
package modules

import (
	logging "cebulka-waf/core"
	"net/http"
	"sort"
)

type ModifierFunc func([]byte, http.Header) []byte

type Module struct {
	Name        string
	Description string
	Version     string
	Priority    int
	Modifier    ModifierFunc
}

var _ModuleRegistry = make(map[string]Module)

func RegisterModule(Mod Module) {
	logging.Log(logging.DebugStatus, "ModuleLoader", "Registered "+Mod.Name)
	_ModuleRegistry[Mod.Name] = Mod
}

func ApplyModules(Content []byte, Headers http.Header) []byte {
	Processed := Content
	var ModList []Module

	for _, V := range _ModuleRegistry {
		ModList = append(ModList, V)
	}

	sort.Slice(ModList, func(I, J int) bool {
		return ModList[I].Priority < ModList[J].Priority
	})

	for _, M := range ModList {
		Processed = M.Modifier(Processed, Headers)
	}

	return Processed
}
