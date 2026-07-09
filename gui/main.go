package main

import (
	_ "embed"
	"os/exec"
	"path/filepath"
	"os"
	"runtime"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/widget"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/canvas"
	"fyne.io/fyne/v2/layout"
)

//go:embed Icon.png
var iconBytes []byte

// findBullarchy returns the path to the bullarchy binary, or "" if not found.
func findBullarchy() string {
	// Check $PATH first
	if path, err := exec.LookPath("bullarchy"); err == nil {
		return path
	}
	// Check ~/.cargo/bin
	home, _ := os.UserHomeDir()
	cargo := filepath.Join(home, ".cargo", "bin", "bullarchy")
	if runtime.GOOS == "windows" {
		cargo += ".exe"
	}
	if _, err := os.Stat(cargo); err == nil {
		return cargo
	}
	return ""
}

func main() {
	a := app.New()
	a.SetIcon(fyne.NewStaticResource("Icon.png", iconBytes))
	w := a.NewWindow("Bullarchy")
	w.Resize(fyne.NewSize(900, 640))

	bullarchyPath := findBullarchy()

	if bullarchyPath == "" {
		// Show error screen
		logo := canvas.NewImageFromResource(fyne.NewStaticResource("Icon.png", iconBytes))
		logo.FillMode = canvas.ImageFillContain
		logo.SetMinSize(fyne.NewSize(80, 80))

		title := widget.NewLabelWithStyle(
			"Bullarchy not found",
			fyne.TextAlignCenter,
			fyne.TextStyle{Bold: true},
		)
		msg := widget.NewLabelWithStyle(
			"The bullarchy CLI is required but was not found on your system.\nInstall it with the command below, then restart this app.",
			fyne.TextAlignCenter,
			fyne.TextStyle{},
		)
		cmd := widget.NewEntry()
		cmd.SetText("cargo install --git https://github.com/The-Bullang-Foundation/Bullarchy.git")
		cmd.Disable()

		copyBtn := widget.NewButtonWithIcon("Copy", theme.ContentCopyIcon(), func() {
			w.Clipboard().SetContent(cmd.Text)
		})

		w.SetContent(container.NewCenter(container.NewVBox(
			container.NewCenter(logo),
			title,
			msg,
			cmd,
			container.NewCenter(copyBtn),
		)))
		w.ShowAndRun()
		return
	}

	// Main UI
	content := buildMainUI(w, bullarchyPath)
	w.SetContent(content)
	w.ShowAndRun()
}

func buildMainUI(w fyne.Window, bin string) fyne.CanvasObject {
	// Sidebar nav buttons
	panels := []struct {
		icon  string
		label string
		build func() fyne.CanvasObject
	}{
		{"⚡", "init",      func() fyne.CanvasObject { return buildInitPanel(bin) }},
		{"🔄", "convert",   func() fyne.CanvasObject { return buildConvertPanel(bin) }},
		{"🗺️", "blueprint", func() fyne.CanvasObject { return buildBlueprintPanel(bin) }},
		{"🔧", "control",   func() fyne.CanvasObject { return buildControlPanel(bin) }},
		{"📦", "packages",  func() fyne.CanvasObject { return buildPackagesPanel(bin) }},
		{"⚙️", "options",   func() fyne.CanvasObject { return buildOptionsPanel(bin) }},
	}

	content := container.NewStack()
	content.Add(buildInitPanel(bin)) // default panel

	var navBtns []*widget.Button
	sidebar := container.NewVBox()

	// Logo at top of sidebar
	logo := canvas.NewImageFromResource(fyne.NewStaticResource("Icon.png", iconBytes))
	logo.FillMode = canvas.ImageFillContain
	logo.SetMinSize(fyne.NewSize(56, 56))
	sidebar.Add(container.NewCenter(logo))
	sidebar.Add(widget.NewSeparator())

	for i, p := range panels {
		i, p := i, p
		btn := widget.NewButton(p.icon+"  "+p.label, func() {
			// Reset all button styles
			for _, b := range navBtns {
				b.Importance = widget.MediumImportance
				b.Refresh()
			}
			navBtns[i].Importance = widget.HighImportance
			navBtns[i].Refresh()
			// Swap panel
			content.Objects = []fyne.CanvasObject{p.build()}
			content.Refresh()
		})
		btn.Alignment = widget.ButtonAlignLeading
		if i == 0 {
			btn.Importance = widget.HighImportance
		}
		navBtns = append(navBtns, btn)
		sidebar.Add(btn)
	}

	sidebar.Add(layout.NewSpacer())

	sideScroll := container.NewVScroll(sidebar)
	sideScroll.SetMinSize(fyne.NewSize(160, 0))

	return container.NewBorder(nil, nil, sideScroll, nil, content)
}
