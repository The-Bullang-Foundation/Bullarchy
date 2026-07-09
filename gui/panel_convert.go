package main

import (
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/widget"
)

func buildConvertPanel(bin string) fyne.CanvasObject {
	targetEntry := widget.NewEntry()
	targetEntry.SetPlaceHolder("./my_project  or  ./file.bu")

	targetBrowse := widget.NewButton("Browse", func() {
		d := dialog.NewFolderOpen(func(uri fyne.ListableURI, err error) {
			if err != nil || uri == nil {
				return
			}
			targetEntry.SetText(uri.Path())
		}, fyne.CurrentApp().Driver().AllWindows()[0])
		d.Show()
	})

	secondEntry := widget.NewEntry()
	secondEntry.SetPlaceHolder("rs / py / c / cpp / go / java  or  out.rs  (optional)")

	out, scroll := consoleOutput()
	btn := runButton("Run convert")

	btn.OnTapped = func() {
		args := []string{"convert"}
		if t := targetEntry.Text; t != "" {
			args = append(args, t)
		}
		if s := secondEntry.Text; s != "" {
			args = append(args, s)
		}
		runBullarchy(bin, out, btn, args...)
	}

	form := container.NewVBox(
		labeledField("Source path", container.NewBorder(nil, nil, nil, targetBrowse, targetEntry)),
		labeledField("Language / Output", secondEntry),
		infoLabel("Short ext = language override (e.g. rs). Filename = explicit output path."),
		btn,
	)

	return container.NewVBox(
		infoLabel("Transpile a Bullang project folder or a single .bu file."),
		widget.NewSeparator(),
		form,
		widget.NewSeparator(),
		scroll,
	)
}
