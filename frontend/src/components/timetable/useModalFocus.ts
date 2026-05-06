import React from "react";

export function useModalFocus(onClose: () => void) {
  const modalRef = React.useRef<HTMLDivElement>(null);
  const previouslyFocused = React.useRef<HTMLElement | null>(null);

  React.useEffect(() => {
    previouslyFocused.current = document.activeElement as HTMLElement;
    const modal = modalRef.current;
    if (!modal) return;

    // Focus the modal container so keyboard events are captured
    modal.focus();

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
        return;
      }
      if (e.key === "Tab") {
        const focusable = Array.from(
          modal.querySelectorAll<HTMLElement>(
            'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
          )
        ).filter((el) => !(el as HTMLElement & { disabled?: boolean }).disabled && el.offsetParent !== null);

        if (focusable.length === 0) return;

        const first = focusable[0];
        const last = focusable[focusable.length - 1];

        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault();
            last.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first.focus();
          }
        }
      }
    };

    modal.addEventListener("keydown", handleKeyDown);
    return () => {
      modal.removeEventListener("keydown", handleKeyDown);
      previouslyFocused.current?.focus();
    };
  }, [onClose]);

  return modalRef;
}
