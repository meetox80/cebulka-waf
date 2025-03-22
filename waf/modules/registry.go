// cebulka-waf/modules/registry.go
package modules

import (
	logging "cebulka-waf/core"
)

type Module struct {
	Name        string
	Description string
	Version     string
	Modifier    func([]byte) []byte
}

var _ModuleRegistry = make(map[string]Module)

func RegisterModule(Mod Module) {
	logging.Log(logging.DebugStatus, "ModuleLoader", "Registered "+Mod.Name)
	_ModuleRegistry[Mod.Name] = Mod
}

func ApplyModules(Content []byte) []byte {
	Processed := Content
	for Key := range _ModuleRegistry {
		Mod := _ModuleRegistry[Key]
		Processed = Mod.Modifier(Processed)
	}
	return Processed
}
