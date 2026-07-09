package main

import (
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/widget"
)

func buildOptionsPanel(bin string) fyne.CanvasObject {
	tabs := container.NewAppTabs(
		container.NewTabItem("🛠️ editor-setup", buildEditorSetupTab(bin)),
		container.NewTabItem("🚀 update",        buildUpdateTab(bin)),
	)
	tabs.SetTabLocation(container.TabLocationTop)
	return tabs
}

func buildEditorSetupTab(bin string) fyne.CanvasObject {
	out, scroll := consoleOutput()
	btn := runButton("Run editor-setup")
	btn.OnTapped = func() {
		runBullarchy(bin, out, btn, "editor-setup")
	}
	return container.NewVBox(
		infoLabel("Write LSP configuration files for Vim, Neovim, Helix, and Emacs automatically.\nFor VSCode: install the extension from the Bullang repo."),
		widget.NewSeparator(),
		btn,
		widget.NewSeparator(),
		scroll,
	)
}

func buildUpdateTab(bin string) fyne.CanvasObject {
	out, scroll := consoleOutput()
	btn := runButton("Update Bullarchy")
	btn.OnTapped = func() {
		runBullarchy(bin, out, btn, "update")
	}
	return container.NewVBox(
		infoLabel("Reinstall Bullarchy from the latest commit on the main branch."),
		widget.NewSeparator(),
		btn,
		widget.NewSeparator(),
		scroll,
	)
}
