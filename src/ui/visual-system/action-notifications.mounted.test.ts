import assert from "node:assert/strict";
import test from "node:test";
import { createElement, useState } from "react";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { ActionNotificationProvider, useActionNotifications, type ActionNoticeSeverity } from "./action-notifications.ts";
import { FormDialog } from "./confirmation-dialog.ts";
import userEvent from "@testing-library/user-event";

function Producer({ severity = "success" }: { severity?: ActionNoticeSeverity }) {
  const notifications = useActionNotifications();
  return createElement("button", { onClick: () => notifications?.publish({ severity, message: "Operación guardada." }) }, "Publicar");
}
function mount(severity: ActionNoticeSeverity = "success") {
  return render(createElement(ActionNotificationProvider, null, createElement(Producer, { severity })));
}
const notices = () => screen.getByRole("region", { name: "Notificaciones de acciones" });

test("nullable hook preserves standalone screen compatibility", () => {
  render(createElement(Producer));
  fireEvent.click(screen.getByRole("button", { name: "Publicar" }));
  assert.equal(screen.queryByRole("region", { name: "Notificaciones de acciones" }), null);
});

test("repeated identical outcomes are distinct, dismissible and do not steal focus", () => {
  mount();
  const publish = screen.getByRole("button", { name: "Publicar" });
  publish.focus();
  fireEvent.click(publish);
  fireEvent.click(publish);
  assert.equal(document.activeElement, publish);
  assert.equal(within(notices()).getAllByText("Operación guardada.").length, 2);
  fireEvent.click(within(notices()).getAllByRole("button", { name: /Cerrar notificación/ })[0]);
  assert.equal(within(notices()).getAllByText("Operación guardada.").length, 1);
  assert.ok(document.querySelector('[data-ui-action-notice][data-severity="success"]'));
});

test("three visible notices retain FIFO overflow until explicit dismissal", () => {
  mount("error");
  for (let i = 0; i < 5; i++) fireEvent.click(screen.getByRole("button", { name: "Publicar" }));
  assert.equal(within(notices()).getAllByRole("button", { name: /Cerrar notificación/ }).length, 3);
  assert.ok(screen.getByText("2 notificaciones en espera"));
  for (let i = 0; i < 5; i++) fireEvent.click(within(notices()).getAllByRole("button", { name: /Cerrar notificación/ })[0]);
  assert.equal(within(notices()).queryAllByRole("button").length, 0);
});

test("success/info expire after six visible seconds; hover and focus independently pause", (t) => {
  t.mock.timers.enable({ apis: ["setTimeout", "Date"] });
  mount("info");
  fireEvent.click(screen.getByRole("button", { name: "Publicar" }));
  const card = document.querySelector<HTMLElement>("[data-ui-action-notice]")!;
  act(() => t.mock.timers.tick(2000));
  fireEvent.mouseEnter(card);
  fireEvent.focus(within(card).getByRole("button"));
  act(() => t.mock.timers.tick(10000));
  fireEvent.mouseLeave(card);
  act(() => t.mock.timers.tick(10000));
  assert.ok(screen.getByText("Operación guardada."));
  fireEvent.blur(within(card).getByRole("button"));
  act(() => t.mock.timers.tick(3999));
  assert.ok(screen.getByText("Operación guardada."));
  act(() => t.mock.timers.tick(1));
  assert.equal(screen.queryByText("Operación guardada."), null);
});

for (const severity of ["error", "warning"] as const) {
  test(`${severity} persists and queued transient notices start their clock only when visible`, (t) => {
    t.mock.timers.enable({ apis: ["setTimeout", "Date"] });
    render(createElement(ActionNotificationProvider, null, createElement(Producer, { severity }), createElement(Producer, { severity: "success" })));
    const buttons = screen.getAllByRole("button", { name: "Publicar" });
    for (let i = 0; i < 3; i++) fireEvent.click(buttons[0]);
    fireEvent.click(buttons[1]);
    act(() => t.mock.timers.tick(60000));
    assert.equal(within(notices()).getAllByRole("button").length, 3);
    fireEvent.click(within(notices()).getAllByRole("button")[0]);
    act(() => t.mock.timers.tick(5999));
    assert.equal(within(notices()).getAllByRole("button").length, 3);
    act(() => t.mock.timers.tick(1));
    assert.equal(within(notices()).getAllByRole("button").length, 2);
  });
}

test("real modal focus containment includes the keyboard close control without publication focus theft", async () => {
  function ModalProducer() {
    return createElement(FormDialog, { open: true, title: "Guardar", description: "Prueba de notificaciones", onCancel: () => undefined }, createElement(Producer, { severity: "error" }));
  }
  render(createElement(ActionNotificationProvider, null, createElement(ModalProducer)));
  const dialog = screen.getByRole("dialog", { name: "Guardar" });
  await waitFor(() => assert.ok(dialog.contains(screen.getByRole("region", { name: "Notificaciones de acciones" }))));
  const publish = within(dialog).getByRole("button", { name: "Publicar" });
  publish.focus();
  fireEvent.click(publish);
  assert.equal(document.activeElement, publish);
  const close = within(dialog).getByRole("button", { name: "Cerrar notificación: Error" });
  act(() => close.focus());
  const user = userEvent.setup({ document });
  await user.keyboard("{Enter}");
  assert.equal(within(dialog).queryByRole("button", { name: "Cerrar notificación: Error" }), null);
});

test("stable live regions move into the active modal and back without losing notices", async () => {
  function ModalExample() {
    const [open, setOpen] = useState(false);
    return createElement("div", null, createElement(Producer),
      createElement("button", { onClick: () => setOpen(!open) }, "Alternar diálogo"),
      open ? createElement("div", { role: "dialog", "aria-modal": true, "aria-label": "Prueba" }, "Contenido") : null);
  }
  mount().unmount();
  render(createElement(ActionNotificationProvider, null, createElement(ModalExample)));
  const polite = screen.getByRole("status");
  const assertive = screen.getByRole("alert");
  fireEvent.click(screen.getByRole("button", { name: "Alternar diálogo" }));
  await waitFor(() => assert.ok(screen.getByRole("dialog").contains(polite)));
  fireEvent.click(screen.getByRole("button", { name: "Publicar" }));
  assert.match(polite.textContent ?? "", /Operación guardada/);
  fireEvent.click(screen.getByRole("button", { name: "Alternar diálogo" }));
  await waitFor(() => assert.equal(polite.closest('[role="dialog"]'), null));
  assert.equal(screen.getByRole("status"), polite);
  assert.equal(screen.getByRole("alert"), assertive);
  assert.ok(screen.getByText("Operación guardada."));
});
