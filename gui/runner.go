package main

import (
	"fmt"
	"os/exec"
	"strings"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/widget"
	"fyne.io/fyne/v2/theme"
)

// consoleOutput creates a scrollable, read-only log widget + scroll container.
func consoleOutput() (*widget.Entry, *container.Scroll) {
	out := widget.NewMultiLineEntry()
	out.Disable()
	out.SetPlaceHolder("Output will appear here...")
	scroll := container.NewScroll(out)
	scroll.SetMinSize(fyne.NewSize(0, 200))
	return out, scroll
}

// appendLog adds a line to the output entry.
func appendLog(out *widget.Entry, msg string) {
	current := out.Text
	if current == "" {
		out.SetText(msg)
	} else {
		out.SetText(current + "\n" + msg)
	}
}

// runBullarchy runs `bullarchy <args...>` asynchronously, streaming output
// to `out`. Disables `btn` during execution and re-enables on completion.
func runBullarchy(bin string, out *widget.Entry, btn *widget.Button, args ...string) {
	btn.Disable()
	out.SetText("")
	go func() {
		defer btn.Enable()
		appendLog(out, fmt.Sprintf("$ bullarchy %s\n", strings.Join(args, " ")))
		cmd := exec.Command(bin, args...)
		cmd.Stdout = &entryWriter{out: out}
		cmd.Stderr = &entryWriter{out: out}
		if err := cmd.Run(); err != nil {
			appendLog(out, fmt.Sprintf("\n✗ %v", err))
		} else {
			appendLog(out, "\n✓ Done.")
		}
	}()
}

type entryWriter struct{ out *widget.Entry }

func (w *entryWriter) Write(p []byte) (n int, err error) {
	for _, line := range strings.Split(strings.TrimRight(string(p), "\n"), "\n") {
		if strings.TrimSpace(line) != "" {
			appendLog(w.out, line)
		}
	}
	return len(p), nil
}

// labeledField returns a VBox with a label above a widget.
func labeledField(label string, w fyne.CanvasObject) *fyne.Container {
	return container.NewVBox(
		widget.NewLabelWithStyle(label, fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		w,
	)
}

// runButton creates a styled primary action button.
func runButton(label string) *widget.Button {
	btn := widget.NewButtonWithIcon(label, theme.MediaPlayIcon(), nil)
	btn.Importance = widget.HighImportance
	return btn
}

// infoLabel creates an italic info/hint label.
func infoLabel(text string) *widget.Label {
	l := widget.NewLabelWithStyle(text, fyne.TextAlignLeading, fyne.TextStyle{Italic: true})
	l.Wrapping = fyne.TextWrapWord
	return l
}
