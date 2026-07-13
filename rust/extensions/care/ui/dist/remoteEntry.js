import { jsx as d, jsxs as R, Fragment as Bt } from "react/jsx-runtime";
import * as u from "react";
import { createContext as kn, useContext as Rn, StrictMode as ni, useState as I, useEffect as Je, useMemo as ri, forwardRef as Lr, createElement as hn, useLayoutEffect as oi } from "react";
import { createRoot as si } from "react-dom/client";
import * as vt from "react-dom";
function ii(e, { id: t, styles: n }, r) {
  const o = e.ownerDocument, s = o.createElement("div");
  if (s.setAttribute("data-ext-root", t), s.className = "h-full w-full", n) {
    const c = o.createElement("style");
    c.setAttribute("data-ext-styles", t), c.textContent = n, s.appendChild(c);
  }
  const i = o.createElement("div");
  i.className = "h-full w-full", s.appendChild(i), e.appendChild(s);
  const a = r(i);
  return () => {
    try {
      typeof a == "function" && a();
    } finally {
      s.remove();
    }
  };
}
const Dr = kn(null);
function ai({ ctx: e, bridge: t, children: n }) {
  return d(Dr.Provider, { value: { ctx: e, call: t.call }, children: n });
}
function Fr(e) {
  const t = Rn(Dr);
  if (!t)
    throw new Error(`${e} called outside a mounted extension remote`);
  return t;
}
function ci() {
  const { ctx: e } = Fr("useSession");
  return e ?? null;
}
function li() {
  return Fr("useMcpClient").call;
}
function Xn(e, t, n, r, o, s) {
  return ii(e, { id: t, styles: n }, (i) => {
    const a = si(i);
    return a.render(d(ni, { children: d(ai, { ctx: r, bridge: o, children: s }) })), () => a.unmount();
  });
}
function di(e) {
  const { id: t, styles: n, page: r, widgets: o = {} } = e;
  return { mount: (a, c, l) => Xn(a, t, n, c, l, r ? r(c, l) : null), mountWidget: (a, c, l, f) => {
    const m = o[f] ?? Object.values(o)[0];
    return Xn(a, t, n, c, l, m ? m(c, l) : null);
  } };
}
const ui = "[data-ext-root]{--color-background: hsl(var(--background));--color-foreground: hsl(var(--foreground));--color-card: hsl(var(--card));--color-card-foreground: hsl(var(--card-foreground));--color-popover: hsl(var(--popover));--color-popover-foreground: hsl(var(--popover-foreground));--color-primary: hsl(var(--primary));--color-primary-foreground: hsl(var(--primary-foreground));--color-secondary: hsl(var(--secondary));--color-secondary-foreground: hsl(var(--secondary-foreground));--color-muted: hsl(var(--muted));--color-muted-foreground: hsl(var(--muted-foreground));--color-accent: hsl(var(--accent));--color-accent-foreground: hsl(var(--accent-foreground));--color-destructive: hsl(var(--destructive));--color-destructive-foreground: hsl(var(--destructive-foreground));--color-border: hsl(var(--border));--color-input: hsl(var(--input));--color-ring: hsl(var(--ring));--care-radius: .75rem;--care-touch-target: 2.75rem}", fi = {
  "app.title": "Childcare",
  "nav.feed": "Today",
  "nav.children": "Children",
  "nav.menus": "Menus",
  "nav.messages": "Messages",
  "nav.admin": "Admin",
  "auth.signIn": "Sign in",
  "auth.signOut": "Sign out",
  "common.save": "Save",
  "common.cancel": "Cancel",
  "common.back": "Back",
  "common.loading": "Loading…",
  "common.error_generic": "Something went wrong. Please try again.",
  "common.yes": "Yes",
  "common.no": "No",
  "common.add": "Add",
  "common.delete": "Remove",
  "common.archive": "Archive",
  "common.restore": "Restore",
  "common.confirm": "Confirm",
  "common.edit": "Edit",
  "common.next": "Next",
  "common.optional": "(optional)",
  "shell.theme.light": "Light",
  "shell.theme.dark": "Dark",
  "shell.theme.toggle": "Theme",
  "child.name": "Name",
  "child.dob": "Date of birth",
  "child.allergies": "Allergies",
  "child.medical_notes": "Medical notes",
  "child.immunizations": "Immunizations",
  "child.photo_consent": "May we share photos?",
  "child.room": "Room",
  "child.emergency_contacts": "Emergency contacts",
  "child.authorized_pickups": "Authorized pickups",
  "child.contact_name": "Contact name",
  "child.contact_phone": "Phone",
  "child.contact_relationship": "Relationship",
  "child.pickup_name": "Pickup person",
  "child.pickup_phone": "Phone",
  "child.editor.title.new": "New child",
  "child.editor.title.edit": "Edit child",
  "child.editor.safety": "Safety data",
  "child.editor.safety_help": "Allergies and medical notes are stored securely and shared only with staff who care for this child.",
  "child.editor.contacts": "Contacts",
  "child.editor.consent": "Photo consent",
  "child.empty": "No children yet. Add one to begin.",
  "child.created": "Child profile saved.",
  "child.required.allergies_hint": "List every allergy — even minor ones.",
  "center.editor.title.new": "New center",
  "center.editor.title.edit": "Edit center",
  "center.list.title": "Centers",
  "center.name": "Name",
  "center.address": "Address",
  "center.phone": "Phone",
  "center.email": "Email",
  "center.default_locale": "Default language",
  "center.locale.en": "English",
  "center.locale.es": "Spanish",
  "center.empty": "No centers yet.",
  "center.created": "Center saved.",
  "room.editor.title.new": "New room",
  "room.editor.title.edit": "Edit room",
  "room.list.title": "Rooms",
  "room.name": "Name",
  "room.center": "Center",
  "room.empty": "No rooms yet.",
  "room.created": "Room saved.",
  "guardian.editor.title.new": "New guardian",
  "guardian.editor.title.edit": "Edit guardian",
  "guardian.list.title": "Guardians",
  "guardian.name": "Name",
  "guardian.email": "Email",
  "guardian.phone": "Phone",
  "guardian.locale": "Preferred language",
  "guardian.empty": "No guardians yet.",
  "guardian.created": "Guardian saved.",
  "guardian.invite_pending": "Invite pending",
  "edge.list.title": "Family & edges",
  "edge.editor.title.new": "Link a guardian",
  "edge.editor.title.edit": "Edit link",
  "edge.relationship": "Relationship",
  "edge.flag.can_pickup": "Can pick up",
  "edge.flag.receives_daily_feed": "Receives daily feed",
  "edge.flag.receives_billing": "Receives billing",
  "edge.flag.emergency_contact": "Emergency contact",
  "edge.flag.custody_notes": "Custody notes",
  "edge.relationship.mother": "Mother",
  "edge.relationship.father": "Father",
  "edge.relationship.grandparent": "Grandparent",
  "edge.relationship.guardian": "Guardian",
  "edge.relationship.other": "Other",
  "edge.linked": "Guardian linked.",
  "edge.unlinked": "Link removed.",
  "edge.empty": "No links yet for this child.",
  "enrollment.list.title": "Enrollments",
  "enrollment.waitlist.title": "Waitlist",
  "enrollment.editor.title.new": "Enroll a child",
  "enrollment.editor.title.edit": "Edit enrollment",
  "enrollment.child": "Child",
  "enrollment.room": "Room",
  "enrollment.status": "Status",
  "enrollment.status.enrolled": "Enrolled",
  "enrollment.status.waitlist": "Waitlist",
  "enrollment.status.withdrawn": "Withdrawn",
  "enrollment.schedule": "Schedule",
  "enrollment.day.mon": "Mon",
  "enrollment.day.tue": "Tue",
  "enrollment.day.wed": "Wed",
  "enrollment.day.thu": "Thu",
  "enrollment.day.fri": "Fri",
  "enrollment.day.sat": "Sat",
  "enrollment.day.sun": "Sun",
  "enrollment.start_date": "Start date",
  "enrollment.position": "Position {{position}}",
  "enrollment.created": "Enrollment saved.",
  "enrollment.waitlist_empty": "No children on this waitlist.",
  "attendance.checkIn": "Check in",
  "attendance.checkOut": "Check out",
  "log.add.title": "Add to today's log",
  "log.type.meal": "Meal",
  "log.type.nap": "Nap",
  "log.type.diaper": "Diaper",
  "log.type.activity": "Activity",
  "log.type.note": "Note",
  "log.type.incident": "Incident",
  "menu.today": "Today's menu",
  "menu.substitutions": "Substitutions",
  "invite.create": "Invite a guardian",
  "invite.accept": "Accept invite",
  "feed.empty": "No updates yet today.",
  "admin.title": "Admin",
  "admin.schools": "Centers & rooms",
  "admin.enrollment": "Enrollment",
  "admin.guardians": "Guardians",
  "error.denied": "You don't have access to this.",
  "error.offline": "You appear to be offline."
}, mi = {
  "app.title": "Cuidado infantil",
  "nav.feed": "Hoy",
  "nav.children": "Niños",
  "nav.menus": "Menús",
  "nav.messages": "Mensajes",
  "nav.admin": "Administración",
  "auth.signIn": "Iniciar sesión",
  "auth.signOut": "Cerrar sesión",
  "common.save": "Guardar",
  "common.cancel": "Cancelar",
  "common.back": "Atrás",
  "common.loading": "Cargando…",
  "common.error_generic": "Algo salió mal. Inténtalo de nuevo.",
  "common.yes": "Sí",
  "common.no": "No",
  "common.add": "Añadir",
  "common.delete": "Quitar",
  "common.archive": "Archivar",
  "common.restore": "Restaurar",
  "common.confirm": "Confirmar",
  "common.edit": "Editar",
  "common.next": "Siguiente",
  "common.optional": "(opcional)",
  "shell.theme.light": "Claro",
  "shell.theme.dark": "Oscuro",
  "shell.theme.toggle": "Tema",
  "child.name": "Nombre",
  "child.dob": "Fecha de nacimiento",
  "child.allergies": "Alergias",
  "child.medical_notes": "Notas médicas",
  "child.immunizations": "Vacunas",
  "child.photo_consent": "¿Podemos compartir fotos?",
  "child.room": "Sala",
  "child.emergency_contacts": "Contactos de emergencia",
  "child.authorized_pickups": "Personas autorizadas a recoger",
  "child.contact_name": "Nombre del contacto",
  "child.contact_phone": "Teléfono",
  "child.contact_relationship": "Relación",
  "child.pickup_name": "Persona que recoge",
  "child.pickup_phone": "Teléfono",
  "child.editor.title.new": "Nuevo niño",
  "child.editor.title.edit": "Editar niño",
  "child.editor.safety": "Datos de seguridad",
  "child.editor.safety_help": "Las alergias y notas médicas se guardan de forma segura y solo las ve el personal que cuida al niño.",
  "child.editor.contacts": "Contactos",
  "child.editor.consent": "Consentimiento de fotos",
  "child.empty": "Aún no hay niños. Añade uno para empezar.",
  "child.created": "Perfil del niño guardado.",
  "child.required.allergies_hint": "Anota todas las alergias, incluso las leves.",
  "center.editor.title.new": "Nuevo centro",
  "center.editor.title.edit": "Editar centro",
  "center.list.title": "Centros",
  "center.name": "Nombre",
  "center.address": "Dirección",
  "center.phone": "Teléfono",
  "center.email": "Correo electrónico",
  "center.default_locale": "Idioma por defecto",
  "center.locale.en": "Inglés",
  "center.locale.es": "Español",
  "center.empty": "Aún no hay centros.",
  "center.created": "Centro guardado.",
  "room.editor.title.new": "Nueva sala",
  "room.editor.title.edit": "Editar sala",
  "room.list.title": "Salas",
  "room.name": "Nombre",
  "room.center": "Centro",
  "room.empty": "Aún no hay salas.",
  "room.created": "Sala guardada.",
  "guardian.editor.title.new": "Nuevo tutor",
  "guardian.editor.title.edit": "Editar tutor",
  "guardian.list.title": "Tutores",
  "guardian.name": "Nombre",
  "guardian.email": "Correo electrónico",
  "guardian.phone": "Teléfono",
  "guardian.locale": "Idioma preferido",
  "guardian.empty": "Aún no hay tutores.",
  "guardian.created": "Tutor guardado.",
  "guardian.invite_pending": "Invitación pendiente",
  "edge.list.title": "Familia y vínculos",
  "edge.editor.title.new": "Vincular tutor",
  "edge.editor.title.edit": "Editar vínculo",
  "edge.relationship": "Relación",
  "edge.flag.can_pickup": "Puede recoger",
  "edge.flag.receives_daily_feed": "Recibe el registro diario",
  "edge.flag.receives_billing": "Recibe facturación",
  "edge.flag.emergency_contact": "Contacto de emergencia",
  "edge.flag.custody_notes": "Notas de custodia",
  "edge.relationship.mother": "Madre",
  "edge.relationship.father": "Padre",
  "edge.relationship.grandparent": "Abuelo/a",
  "edge.relationship.guardian": "Tutor legal",
  "edge.relationship.other": "Otro",
  "edge.linked": "Tutor vinculado.",
  "edge.unlinked": "Vínculo eliminado.",
  "edge.empty": "Aún no hay vínculos para este niño.",
  "enrollment.list.title": "Inscripciones",
  "enrollment.waitlist.title": "Lista de espera",
  "enrollment.editor.title.new": "Inscribir un niño",
  "enrollment.editor.title.edit": "Editar inscripción",
  "enrollment.child": "Niño",
  "enrollment.room": "Sala",
  "enrollment.status": "Estado",
  "enrollment.status.enrolled": "Inscrito",
  "enrollment.status.waitlist": "Lista de espera",
  "enrollment.status.withdrawn": "Retirado",
  "enrollment.schedule": "Horario",
  "enrollment.day.mon": "Lun",
  "enrollment.day.tue": "Mar",
  "enrollment.day.wed": "Mié",
  "enrollment.day.thu": "Jue",
  "enrollment.day.fri": "Vie",
  "enrollment.day.sat": "Sáb",
  "enrollment.day.sun": "Dom",
  "enrollment.start_date": "Fecha de inicio",
  "enrollment.position": "Posición {{position}}",
  "enrollment.created": "Inscripción guardada.",
  "enrollment.waitlist_empty": "No hay niños en esta lista de espera.",
  "attendance.checkIn": "Entrada",
  "attendance.checkOut": "Salida",
  "log.add.title": "Añadir al registro de hoy",
  "log.type.meal": "Comida",
  "log.type.nap": "Siesta",
  "log.type.diaper": "Pañal",
  "log.type.activity": "Actividad",
  "log.type.note": "Nota",
  "log.type.incident": "Incidente",
  "menu.today": "Menú de hoy",
  "menu.substitutions": "Sustituciones",
  "invite.create": "Invitar a un tutor",
  "invite.accept": "Aceptar invitación",
  "feed.empty": "Aún no hay actualizaciones hoy.",
  "admin.title": "Administración",
  "admin.schools": "Centros y salas",
  "admin.enrollment": "Inscripciones",
  "admin.guardians": "Tutores",
  "error.denied": "No tienes acceso a esto.",
  "error.offline": "Parece que estás sin conexión."
}, Zn = { en: fi, es: mi };
function pi(e, t) {
  return t ? e.replace(/\{\{(\w+)\}\}/g, (n, r) => r in t ? String(t[r]) : `{{${r}}}`) : e;
}
const zr = kn(null);
function en({ children: e }) {
  const [t, n] = I("en"), [r, o] = I("system"), [s, i] = I(!1);
  Je(() => {
    const l = localStorage.getItem("care.locale");
    (l === "en" || l === "es") && n(l);
    const f = localStorage.getItem("care.theme");
    (f === "light" || f === "dark" || f === "system") && o(f);
    const m = window.matchMedia("(prefers-color-scheme: dark)");
    i(m.matches);
    const h = (g) => i(g.matches);
    return m.addEventListener("change", h), () => m.removeEventListener("change", h);
  }, []);
  const a = r === "system" ? s ? "dark" : "light" : r;
  Je(() => {
    document.documentElement.classList.toggle("dark", a === "dark");
  }, [a]);
  const c = ri(() => ({
    locale: t,
    setLocale: (l) => {
      localStorage.setItem("care.locale", l), n(l);
    },
    t: (l, f) => pi(Zn[t][l] ?? Zn.en[l] ?? l, f),
    theme: r,
    setTheme: (l) => {
      localStorage.setItem("care.theme", l), o(l);
    },
    resolvedTheme: a
  }), [t, r, a]);
  return /* @__PURE__ */ d(zr.Provider, { value: c, children: e });
}
function fe() {
  const e = Rn(zr);
  if (!e) throw new Error("useT outside LocaleProvider");
  return e.t;
}
function he({ children: e, trailing: t }) {
  return /* @__PURE__ */ R("header", { className: "flex items-end justify-between gap-3 px-4 pb-2 pt-3", children: [
    /* @__PURE__ */ d("h1", { className: "text-[1.75rem] font-bold leading-tight tracking-tight text-foreground", children: e }),
    t
  ] });
}
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const $r = (...e) => e.filter((t, n, r) => !!t && t.trim() !== "" && r.indexOf(t) === n).join(" ").trim();
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const hi = (e) => e.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase();
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const gi = (e) => e.replace(
  /^([A-Z])|[\s-_]+(\w)/g,
  (t, n, r) => r ? r.toUpperCase() : n.toLowerCase()
);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Qn = (e) => {
  const t = gi(e);
  return t.charAt(0).toUpperCase() + t.slice(1);
};
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
var tn = {
  xmlns: "http://www.w3.org/2000/svg",
  width: 24,
  height: 24,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round",
  strokeLinejoin: "round"
};
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const vi = (e) => {
  for (const t in e)
    if (t.startsWith("aria-") || t === "role" || t === "title")
      return !0;
  return !1;
}, bi = kn({}), yi = () => Rn(bi), wi = Lr(
  ({ color: e, size: t, strokeWidth: n, absoluteStrokeWidth: r, className: o = "", children: s, iconNode: i, ...a }, c) => {
    const {
      size: l = 24,
      strokeWidth: f = 2,
      absoluteStrokeWidth: m = !1,
      color: h = "currentColor",
      className: g = ""
    } = yi() ?? {}, w = r ?? m ? Number(n ?? f) * 24 / Number(t ?? l) : n ?? f;
    return hn(
      "svg",
      {
        ref: c,
        ...tn,
        width: t ?? l ?? tn.width,
        height: t ?? l ?? tn.height,
        stroke: e ?? h,
        strokeWidth: w,
        className: $r("lucide", g, o),
        ...!s && !vi(a) && { "aria-hidden": "true" },
        ...a
      },
      [
        ...i.map(([p, x]) => hn(p, x)),
        ...Array.isArray(s) ? s : [s]
      ]
    );
  }
);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Pe = (e, t) => {
  const n = Lr(
    ({ className: r, ...o }, s) => hn(wi, {
      ref: s,
      iconNode: t,
      className: $r(
        `lucide-${hi(Qn(e))}`,
        `lucide-${e}`,
        r
      ),
      ...o
    })
  );
  return n.displayName = Qn(e), n;
};
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const xi = [["path", { d: "M20 6 9 17l-5-5", key: "1gmf2c" }]], Ci = Pe("check", xi);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Si = [["path", { d: "m15 18-6-6 6-6", key: "1wnfg3" }]], Ni = Pe("chevron-left", Si);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ei = [["path", { d: "m6 9 6 6 6-6", key: "qrunsl" }]], ki = Pe("chevron-down", Ei);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ri = [["path", { d: "m9 18 6-6-6-6", key: "mthhwq" }]], Pn = Pe("chevron-right", Ri);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Pi = [
  ["path", { d: "M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8", key: "5wwlr5" }],
  [
    "path",
    {
      d: "M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
      key: "r6nss1"
    }
  ]
], Ai = Pe("house", Pi);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ti = [
  ["path", { d: "M5 12h14", key: "1ays0h" }],
  ["path", { d: "M12 5v14", key: "s699le" }]
], et = Pe("plus", Ti);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const _i = [
  [
    "path",
    {
      d: "M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915",
      key: "1i5ecw"
    }
  ],
  ["circle", { cx: "12", cy: "12", r: "3", key: "1v7zrd" }]
], Ii = Pe("settings", _i);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Oi = [
  [
    "path",
    {
      d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3",
      key: "wmoenq"
    }
  ],
  ["path", { d: "M12 9v4", key: "juzpu7" }],
  ["path", { d: "M12 17h.01", key: "p32p05" }]
], Mi = Pe("triangle-alert", Oi);
/**
 * @license lucide-react v1.24.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Li = [
  ["path", { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2", key: "1yyitq" }],
  ["path", { d: "M16 3.128a4 4 0 0 1 0 7.744", key: "16gr8j" }],
  ["path", { d: "M22 21v-2a4 4 0 0 0-3-3.87", key: "kshegd" }],
  ["circle", { cx: "9", cy: "7", r: "4", key: "nufk8" }]
], Di = Pe("users", Li);
function Wr() {
  return ci() ?? null;
}
function Br(e) {
  var t, n, r = "";
  if (typeof e == "string" || typeof e == "number") r += e;
  else if (typeof e == "object") if (Array.isArray(e)) {
    var o = e.length;
    for (t = 0; t < o; t++) e[t] && (n = Br(e[t])) && (r && (r += " "), r += n);
  } else for (n in e) e[n] && (r && (r += " "), r += n);
  return r;
}
function Vr() {
  for (var e, t, n = 0, r = "", o = arguments.length; n < o; n++) (e = arguments[n]) && (t = Br(e)) && (r && (r += " "), r += t);
  return r;
}
const Fi = (e, t) => {
  const n = new Array(e.length + t.length);
  for (let r = 0; r < e.length; r++)
    n[r] = e[r];
  for (let r = 0; r < t.length; r++)
    n[e.length + r] = t[r];
  return n;
}, zi = (e, t) => ({
  classGroupId: e,
  validator: t
}), Hr = (e = /* @__PURE__ */ new Map(), t = null, n) => ({
  nextPart: e,
  validators: t,
  classGroupId: n
}), Ot = "-", Jn = [], $i = "arbitrary..", Wi = (e) => {
  const t = Vi(e), {
    conflictingClassGroups: n,
    conflictingClassGroupModifiers: r
  } = e;
  return {
    getClassGroupId: (i) => {
      if (i.startsWith("[") && i.endsWith("]"))
        return Bi(i);
      const a = i.split(Ot), c = a[0] === "" && a.length > 1 ? 1 : 0;
      return Ur(a, c, t);
    },
    getConflictingClassGroupIds: (i, a) => {
      if (a) {
        const c = r[i], l = n[i];
        return c ? l ? Fi(l, c) : c : l || Jn;
      }
      return n[i] || Jn;
    }
  };
}, Ur = (e, t, n) => {
  if (e.length - t === 0)
    return n.classGroupId;
  const o = e[t], s = n.nextPart.get(o);
  if (s) {
    const l = Ur(e, t + 1, s);
    if (l) return l;
  }
  const i = n.validators;
  if (i === null)
    return;
  const a = t === 0 ? e.join(Ot) : e.slice(t).join(Ot), c = i.length;
  for (let l = 0; l < c; l++) {
    const f = i[l];
    if (f.validator(a))
      return f.classGroupId;
  }
}, Bi = (e) => e.slice(1, -1).indexOf(":") === -1 ? void 0 : (() => {
  const t = e.slice(1, -1), n = t.indexOf(":"), r = t.slice(0, n);
  return r ? $i + r : void 0;
})(), Vi = (e) => {
  const {
    theme: t,
    classGroups: n
  } = e;
  return Hi(n, t);
}, Hi = (e, t) => {
  const n = Hr();
  for (const r in e) {
    const o = e[r];
    An(o, n, r, t);
  }
  return n;
}, An = (e, t, n, r) => {
  const o = e.length;
  for (let s = 0; s < o; s++) {
    const i = e[s];
    Ui(i, t, n, r);
  }
}, Ui = (e, t, n, r) => {
  if (typeof e == "string") {
    ji(e, t, n);
    return;
  }
  if (typeof e == "function") {
    Gi(e, t, n, r);
    return;
  }
  Ki(e, t, n, r);
}, ji = (e, t, n) => {
  const r = e === "" ? t : jr(t, e);
  r.classGroupId = n;
}, Gi = (e, t, n, r) => {
  if (qi(e)) {
    An(e(r), t, n, r);
    return;
  }
  t.validators === null && (t.validators = []), t.validators.push(zi(n, e));
}, Ki = (e, t, n, r) => {
  const o = Object.entries(e), s = o.length;
  for (let i = 0; i < s; i++) {
    const [a, c] = o[i];
    An(c, jr(t, a), n, r);
  }
}, jr = (e, t) => {
  let n = e;
  const r = t.split(Ot), o = r.length;
  for (let s = 0; s < o; s++) {
    const i = r[s];
    let a = n.nextPart.get(i);
    a || (a = Hr(), n.nextPart.set(i, a)), n = a;
  }
  return n;
}, qi = (e) => "isThemeGetter" in e && e.isThemeGetter === !0, Yi = (e) => {
  if (e < 1)
    return {
      get: () => {
      },
      set: () => {
      }
    };
  let t = 0, n = /* @__PURE__ */ Object.create(null), r = /* @__PURE__ */ Object.create(null);
  const o = (s, i) => {
    n[s] = i, t++, t > e && (t = 0, r = n, n = /* @__PURE__ */ Object.create(null));
  };
  return {
    get(s) {
      let i = n[s];
      if (i !== void 0)
        return i;
      if ((i = r[s]) !== void 0)
        return o(s, i), i;
    },
    set(s, i) {
      s in n ? n[s] = i : o(s, i);
    }
  };
}, gn = "!", er = ":", Xi = [], tr = (e, t, n, r, o) => ({
  modifiers: e,
  hasImportantModifier: t,
  baseClassName: n,
  maybePostfixModifierPosition: r,
  isExternal: o
}), Zi = (e) => {
  const {
    prefix: t,
    experimentalParseClassName: n
  } = e;
  let r = (o) => {
    const s = [];
    let i = 0, a = 0, c = 0, l;
    const f = o.length;
    for (let p = 0; p < f; p++) {
      const x = o[p];
      if (i === 0 && a === 0) {
        if (x === er) {
          s.push(o.slice(c, p)), c = p + 1;
          continue;
        }
        if (x === "/") {
          l = p;
          continue;
        }
      }
      x === "[" ? i++ : x === "]" ? i-- : x === "(" ? a++ : x === ")" && a--;
    }
    const m = s.length === 0 ? o : o.slice(c);
    let h = m, g = !1;
    m.endsWith(gn) ? (h = m.slice(0, -1), g = !0) : (
      /**
       * In Tailwind CSS v3 the important modifier was at the start of the base class name. This is still supported for legacy reasons.
       * @see https://github.com/dcastil/tailwind-merge/issues/513#issuecomment-2614029864
       */
      m.startsWith(gn) && (h = m.slice(1), g = !0)
    );
    const w = l && l > c ? l - c : void 0;
    return tr(s, g, h, w);
  };
  if (t) {
    const o = t + er, s = r;
    r = (i) => i.startsWith(o) ? s(i.slice(o.length)) : tr(Xi, !1, i, void 0, !0);
  }
  if (n) {
    const o = r;
    r = (s) => n({
      className: s,
      parseClassName: o
    });
  }
  return r;
}, Qi = (e) => {
  const t = /* @__PURE__ */ new Map();
  return e.orderSensitiveModifiers.forEach((n, r) => {
    t.set(n, 1e6 + r);
  }), (n) => {
    const r = [];
    let o = [];
    for (let s = 0; s < n.length; s++) {
      const i = n[s], a = i[0] === "[", c = t.has(i);
      a || c ? (o.length > 0 && (o.sort(), r.push(...o), o = []), r.push(i)) : o.push(i);
    }
    return o.length > 0 && (o.sort(), r.push(...o)), r;
  };
}, Ji = (e) => ({
  cache: Yi(e.cacheSize),
  parseClassName: Zi(e),
  sortModifiers: Qi(e),
  postfixLookupClassGroupIds: ea(e),
  ...Wi(e)
}), ea = (e) => {
  const t = /* @__PURE__ */ Object.create(null), n = e.postfixLookupClassGroups;
  if (n)
    for (let r = 0; r < n.length; r++)
      t[n[r]] = !0;
  return t;
}, ta = /\s+/, na = (e, t) => {
  const {
    parseClassName: n,
    getClassGroupId: r,
    getConflictingClassGroupIds: o,
    sortModifiers: s,
    postfixLookupClassGroupIds: i
  } = t, a = [], c = e.trim().split(ta);
  let l = "";
  for (let f = c.length - 1; f >= 0; f -= 1) {
    const m = c[f], {
      isExternal: h,
      modifiers: g,
      hasImportantModifier: w,
      baseClassName: p,
      maybePostfixModifierPosition: x
    } = n(m);
    if (h) {
      l = m + (l.length > 0 ? " " + l : l);
      continue;
    }
    let b = !!x, y;
    if (b) {
      const k = p.substring(0, x);
      y = r(k);
      const N = y && i[y] ? r(p) : void 0;
      N && N !== y && (y = N, b = !1);
    } else
      y = r(p);
    if (!y) {
      if (!b) {
        l = m + (l.length > 0 ? " " + l : l);
        continue;
      }
      if (y = r(p), !y) {
        l = m + (l.length > 0 ? " " + l : l);
        continue;
      }
      b = !1;
    }
    const v = g.length === 0 ? "" : g.length === 1 ? g[0] : s(g).join(":"), S = w ? v + gn : v, E = S + y;
    if (a.indexOf(E) > -1)
      continue;
    a.push(E);
    const C = o(y, b);
    for (let k = 0; k < C.length; ++k) {
      const N = C[k];
      a.push(S + N);
    }
    l = m + (l.length > 0 ? " " + l : l);
  }
  return l;
}, ra = (...e) => {
  let t = 0, n, r, o = "";
  for (; t < e.length; )
    (n = e[t++]) && (r = Gr(n)) && (o && (o += " "), o += r);
  return o;
}, Gr = (e) => {
  if (typeof e == "string")
    return e;
  let t, n = "";
  for (let r = 0; r < e.length; r++)
    e[r] && (t = Gr(e[r])) && (n && (n += " "), n += t);
  return n;
}, oa = (e, ...t) => {
  let n, r, o, s;
  const i = (c) => {
    const l = t.reduce((f, m) => m(f), e());
    return n = Ji(l), r = n.cache.get, o = n.cache.set, s = a, a(c);
  }, a = (c) => {
    const l = r(c);
    if (l)
      return l;
    const f = na(c, n);
    return o(c, f), f;
  };
  return s = i, (...c) => s(ra(...c));
}, sa = [], oe = (e) => {
  const t = (n) => n[e] || sa;
  return t.isThemeGetter = !0, t;
}, Kr = /^\[(?:(\w[\w-]*):)?(.+)\]$/i, qr = /^\((?:(\w[\w-]*):)?(.+)\)$/i, ia = /^\d+(?:\.\d+)?\/\d+(?:\.\d+)?$/, aa = /^(\d+(\.\d+)?)?(xs|sm|md|lg|xl)$/, ca = /\d+(%|px|r?em|[sdl]?v([hwib]|min|max)|pt|pc|in|cm|mm|cap|ch|ex|r?lh|cq(w|h|i|b|min|max))|\b(calc|min|max|clamp)\(.+\)|^0$/, la = /^(rgba?|hsla?|hwb|(ok)?(lab|lch)|color-mix)\(.+\)$/, da = /^(inset_)?-?((\d+)?\.?(\d+)[a-z]+|0)_-?((\d+)?\.?(\d+)[a-z]+|0)/, ua = /^(url|image|image-set|cross-fade|element|(repeating-)?(linear|radial|conic)-gradient)\(.+\)$/, _e = (e) => ia.test(e), z = (e) => !!e && !Number.isNaN(Number(e)), ve = (e) => !!e && Number.isInteger(Number(e)), nn = (e) => e.endsWith("%") && z(e.slice(0, -1)), Ne = (e) => aa.test(e), Yr = () => !0, fa = (e) => (
  // `colorFunctionRegex` check is necessary because color functions can have percentages in them which which would be incorrectly classified as lengths.
  // For example, `hsl(0 0% 0%)` would be classified as a length without this check.
  // I could also use lookbehind assertion in `lengthUnitRegex` but that isn't supported widely enough.
  ca.test(e) && !la.test(e)
), Tn = () => !1, ma = (e) => da.test(e), pa = (e) => ua.test(e), ha = (e) => !A(e) && !T(e), ga = (e) => e.startsWith("@container") && (e[10] === "/" && e[11] !== void 0 || e[11] === "s" && e[16] !== void 0 && e.startsWith("-size/", 10) || e[11] === "n" && e[18] !== void 0 && e.startsWith("-normal/", 10)), va = (e) => De(e, Qr, Tn), A = (e) => Kr.test(e), We = (e) => De(e, Jr, fa), nr = (e) => De(e, Ea, z), ba = (e) => De(e, to, Yr), ya = (e) => De(e, eo, Tn), rr = (e) => De(e, Xr, Tn), wa = (e) => De(e, Zr, pa), xt = (e) => De(e, no, ma), T = (e) => qr.test(e), at = (e) => je(e, Jr), xa = (e) => je(e, eo), or = (e) => je(e, Xr), Ca = (e) => je(e, Qr), Sa = (e) => je(e, Zr), Ct = (e) => je(e, no, !0), Na = (e) => je(e, to, !0), De = (e, t, n) => {
  const r = Kr.exec(e);
  return r ? r[1] ? t(r[1]) : n(r[2]) : !1;
}, je = (e, t, n = !1) => {
  const r = qr.exec(e);
  return r ? r[1] ? t(r[1]) : n : !1;
}, Xr = (e) => e === "position" || e === "percentage", Zr = (e) => e === "image" || e === "url", Qr = (e) => e === "length" || e === "size" || e === "bg-size", Jr = (e) => e === "length", Ea = (e) => e === "number", eo = (e) => e === "family-name", to = (e) => e === "number" || e === "weight", no = (e) => e === "shadow", ka = () => {
  const e = oe("color"), t = oe("font"), n = oe("text"), r = oe("font-weight"), o = oe("tracking"), s = oe("leading"), i = oe("breakpoint"), a = oe("container"), c = oe("spacing"), l = oe("radius"), f = oe("shadow"), m = oe("inset-shadow"), h = oe("text-shadow"), g = oe("drop-shadow"), w = oe("blur"), p = oe("perspective"), x = oe("aspect"), b = oe("ease"), y = oe("animate"), v = () => ["auto", "avoid", "all", "avoid-page", "page", "left", "right", "column"], S = () => [
    "center",
    "top",
    "bottom",
    "left",
    "right",
    "top-left",
    // Deprecated since Tailwind CSS v4.1.0, see https://github.com/tailwindlabs/tailwindcss/pull/17378
    "left-top",
    "top-right",
    // Deprecated since Tailwind CSS v4.1.0, see https://github.com/tailwindlabs/tailwindcss/pull/17378
    "right-top",
    "bottom-right",
    // Deprecated since Tailwind CSS v4.1.0, see https://github.com/tailwindlabs/tailwindcss/pull/17378
    "right-bottom",
    "bottom-left",
    // Deprecated since Tailwind CSS v4.1.0, see https://github.com/tailwindlabs/tailwindcss/pull/17378
    "left-bottom"
  ], E = () => [...S(), T, A], C = () => ["auto", "hidden", "clip", "visible", "scroll"], k = () => ["auto", "contain", "none"], N = () => [T, A, c], L = () => [_e, "full", "auto", ...N()], F = () => [ve, "none", "subgrid", T, A], _ = () => ["auto", {
    span: ["full", ve, T, A]
  }, ve, T, A], B = () => [ve, "auto", T, A], $ = () => ["auto", "min", "max", "fr", T, A], H = () => ["start", "end", "center", "between", "around", "evenly", "stretch", "baseline", "center-safe", "end-safe"], U = () => ["start", "end", "center", "stretch", "center-safe", "end-safe"], O = () => ["auto", ...N()], G = () => [_e, "auto", "full", "dvw", "dvh", "lvw", "lvh", "svw", "svh", "min", "max", "fit", ...N()], M = () => [_e, "screen", "full", "dvw", "lvw", "svw", "min", "max", "fit", ...N()], K = () => [_e, "screen", "full", "lh", "dvh", "lvh", "svh", "min", "max", "fit", ...N()], P = () => [e, T, A], me = () => [...S(), or, rr, {
    position: [T, A]
  }], ee = () => ["no-repeat", {
    repeat: ["", "x", "y", "space", "round"]
  }], ce = () => ["auto", "cover", "contain", Ca, va, {
    size: [T, A]
  }], le = () => [nn, at, We], J = () => [
    // Deprecated since Tailwind CSS v4.0.0
    "",
    "none",
    "full",
    l,
    T,
    A
  ], Q = () => ["", z, at, We], D = () => ["solid", "dashed", "dotted", "double"], Z = () => ["normal", "multiply", "screen", "overlay", "darken", "lighten", "color-dodge", "color-burn", "hard-light", "soft-light", "difference", "exclusion", "hue", "saturation", "color", "luminosity"], V = () => [z, nn, or, rr], X = () => [
    // Deprecated since Tailwind CSS v4.0.0
    "",
    "none",
    w,
    T,
    A
  ], W = () => ["none", z, T, A], j = () => ["none", z, T, A], de = () => [z, T, A], ae = () => [_e, "full", ...N()];
  return {
    cacheSize: 500,
    theme: {
      animate: ["spin", "ping", "pulse", "bounce"],
      aspect: ["video"],
      blur: [Ne],
      breakpoint: [Ne],
      color: [Yr],
      container: [Ne],
      "drop-shadow": [Ne],
      ease: ["in", "out", "in-out"],
      font: [ha],
      "font-weight": ["thin", "extralight", "light", "normal", "medium", "semibold", "bold", "extrabold", "black"],
      "inset-shadow": [Ne],
      leading: ["none", "tight", "snug", "normal", "relaxed", "loose"],
      perspective: ["dramatic", "near", "normal", "midrange", "distant", "none"],
      radius: [Ne],
      shadow: [Ne],
      spacing: ["px", z],
      text: [Ne],
      "text-shadow": [Ne],
      tracking: ["tighter", "tight", "normal", "wide", "wider", "widest"]
    },
    classGroups: {
      // --------------
      // --- Layout ---
      // --------------
      /**
       * Aspect Ratio
       * @see https://tailwindcss.com/docs/aspect-ratio
       */
      aspect: [{
        aspect: ["auto", "square", _e, A, T, x]
      }],
      /**
       * Container
       * @see https://tailwindcss.com/docs/container
       * @deprecated since Tailwind CSS v4.0.0
       */
      container: ["container"],
      /**
       * Container Type
       * @see https://tailwindcss.com/docs/responsive-design#container-queries
       */
      "container-type": [{
        "@container": ["", "normal", "size", T, A]
      }],
      /**
       * Container Name
       * @see https://tailwindcss.com/docs/responsive-design#named-containers
       */
      "container-named": [ga],
      /**
       * Columns
       * @see https://tailwindcss.com/docs/columns
       */
      columns: [{
        columns: [z, A, T, a]
      }],
      /**
       * Break After
       * @see https://tailwindcss.com/docs/break-after
       */
      "break-after": [{
        "break-after": v()
      }],
      /**
       * Break Before
       * @see https://tailwindcss.com/docs/break-before
       */
      "break-before": [{
        "break-before": v()
      }],
      /**
       * Break Inside
       * @see https://tailwindcss.com/docs/break-inside
       */
      "break-inside": [{
        "break-inside": ["auto", "avoid", "avoid-page", "avoid-column"]
      }],
      /**
       * Box Decoration Break
       * @see https://tailwindcss.com/docs/box-decoration-break
       */
      "box-decoration": [{
        "box-decoration": ["slice", "clone"]
      }],
      /**
       * Box Sizing
       * @see https://tailwindcss.com/docs/box-sizing
       */
      box: [{
        box: ["border", "content"]
      }],
      /**
       * Display
       * @see https://tailwindcss.com/docs/display
       */
      display: ["block", "inline-block", "inline", "flex", "inline-flex", "table", "inline-table", "table-caption", "table-cell", "table-column", "table-column-group", "table-footer-group", "table-header-group", "table-row-group", "table-row", "flow-root", "grid", "inline-grid", "contents", "list-item", "hidden"],
      /**
       * Screen Reader Only
       * @see https://tailwindcss.com/docs/display#screen-reader-only
       */
      sr: ["sr-only", "not-sr-only"],
      /**
       * Floats
       * @see https://tailwindcss.com/docs/float
       */
      float: [{
        float: ["right", "left", "none", "start", "end"]
      }],
      /**
       * Clear
       * @see https://tailwindcss.com/docs/clear
       */
      clear: [{
        clear: ["left", "right", "both", "none", "start", "end"]
      }],
      /**
       * Isolation
       * @see https://tailwindcss.com/docs/isolation
       */
      isolation: ["isolate", "isolation-auto"],
      /**
       * Object Fit
       * @see https://tailwindcss.com/docs/object-fit
       */
      "object-fit": [{
        object: ["contain", "cover", "fill", "none", "scale-down"]
      }],
      /**
       * Object Position
       * @see https://tailwindcss.com/docs/object-position
       */
      "object-position": [{
        object: E()
      }],
      /**
       * Overflow
       * @see https://tailwindcss.com/docs/overflow
       */
      overflow: [{
        overflow: C()
      }],
      /**
       * Overflow X
       * @see https://tailwindcss.com/docs/overflow
       */
      "overflow-x": [{
        "overflow-x": C()
      }],
      /**
       * Overflow Y
       * @see https://tailwindcss.com/docs/overflow
       */
      "overflow-y": [{
        "overflow-y": C()
      }],
      /**
       * Overscroll Behavior
       * @see https://tailwindcss.com/docs/overscroll-behavior
       */
      overscroll: [{
        overscroll: k()
      }],
      /**
       * Overscroll Behavior X
       * @see https://tailwindcss.com/docs/overscroll-behavior
       */
      "overscroll-x": [{
        "overscroll-x": k()
      }],
      /**
       * Overscroll Behavior Y
       * @see https://tailwindcss.com/docs/overscroll-behavior
       */
      "overscroll-y": [{
        "overscroll-y": k()
      }],
      /**
       * Position
       * @see https://tailwindcss.com/docs/position
       */
      position: ["static", "fixed", "absolute", "relative", "sticky"],
      /**
       * Inset
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      inset: [{
        inset: L()
      }],
      /**
       * Inset Inline
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      "inset-x": [{
        "inset-x": L()
      }],
      /**
       * Inset Block
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      "inset-y": [{
        "inset-y": L()
      }],
      /**
       * Inset Inline Start
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       * @todo class group will be renamed to `inset-s` in next major release
       */
      start: [{
        "inset-s": L(),
        /**
         * @deprecated since Tailwind CSS v4.2.0 in favor of `inset-s-*` utilities.
         * @see https://github.com/tailwindlabs/tailwindcss/pull/19613
         */
        start: L()
      }],
      /**
       * Inset Inline End
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       * @todo class group will be renamed to `inset-e` in next major release
       */
      end: [{
        "inset-e": L(),
        /**
         * @deprecated since Tailwind CSS v4.2.0 in favor of `inset-e-*` utilities.
         * @see https://github.com/tailwindlabs/tailwindcss/pull/19613
         */
        end: L()
      }],
      /**
       * Inset Block Start
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      "inset-bs": [{
        "inset-bs": L()
      }],
      /**
       * Inset Block End
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      "inset-be": [{
        "inset-be": L()
      }],
      /**
       * Top
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      top: [{
        top: L()
      }],
      /**
       * Right
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      right: [{
        right: L()
      }],
      /**
       * Bottom
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      bottom: [{
        bottom: L()
      }],
      /**
       * Left
       * @see https://tailwindcss.com/docs/top-right-bottom-left
       */
      left: [{
        left: L()
      }],
      /**
       * Visibility
       * @see https://tailwindcss.com/docs/visibility
       */
      visibility: ["visible", "invisible", "collapse"],
      /**
       * Z-Index
       * @see https://tailwindcss.com/docs/z-index
       */
      z: [{
        z: [ve, "auto", T, A]
      }],
      // ------------------------
      // --- Flexbox and Grid ---
      // ------------------------
      /**
       * Flex Basis
       * @see https://tailwindcss.com/docs/flex-basis
       */
      basis: [{
        basis: [_e, "full", "auto", a, ...N()]
      }],
      /**
       * Flex Direction
       * @see https://tailwindcss.com/docs/flex-direction
       */
      "flex-direction": [{
        flex: ["row", "row-reverse", "col", "col-reverse"]
      }],
      /**
       * Flex Wrap
       * @see https://tailwindcss.com/docs/flex-wrap
       */
      "flex-wrap": [{
        flex: ["nowrap", "wrap", "wrap-reverse"]
      }],
      /**
       * Flex
       * @see https://tailwindcss.com/docs/flex
       */
      flex: [{
        flex: [z, _e, "auto", "initial", "none", A]
      }],
      /**
       * Flex Grow
       * @see https://tailwindcss.com/docs/flex-grow
       */
      grow: [{
        grow: ["", z, T, A]
      }],
      /**
       * Flex Shrink
       * @see https://tailwindcss.com/docs/flex-shrink
       */
      shrink: [{
        shrink: ["", z, T, A]
      }],
      /**
       * Order
       * @see https://tailwindcss.com/docs/order
       */
      order: [{
        order: [ve, "first", "last", "none", T, A]
      }],
      /**
       * Grid Template Columns
       * @see https://tailwindcss.com/docs/grid-template-columns
       */
      "grid-cols": [{
        "grid-cols": F()
      }],
      /**
       * Grid Column Start / End
       * @see https://tailwindcss.com/docs/grid-column
       */
      "col-start-end": [{
        col: _()
      }],
      /**
       * Grid Column Start
       * @see https://tailwindcss.com/docs/grid-column
       */
      "col-start": [{
        "col-start": B()
      }],
      /**
       * Grid Column End
       * @see https://tailwindcss.com/docs/grid-column
       */
      "col-end": [{
        "col-end": B()
      }],
      /**
       * Grid Template Rows
       * @see https://tailwindcss.com/docs/grid-template-rows
       */
      "grid-rows": [{
        "grid-rows": F()
      }],
      /**
       * Grid Row Start / End
       * @see https://tailwindcss.com/docs/grid-row
       */
      "row-start-end": [{
        row: _()
      }],
      /**
       * Grid Row Start
       * @see https://tailwindcss.com/docs/grid-row
       */
      "row-start": [{
        "row-start": B()
      }],
      /**
       * Grid Row End
       * @see https://tailwindcss.com/docs/grid-row
       */
      "row-end": [{
        "row-end": B()
      }],
      /**
       * Grid Auto Flow
       * @see https://tailwindcss.com/docs/grid-auto-flow
       */
      "grid-flow": [{
        "grid-flow": ["row", "col", "dense", "row-dense", "col-dense"]
      }],
      /**
       * Grid Auto Columns
       * @see https://tailwindcss.com/docs/grid-auto-columns
       */
      "auto-cols": [{
        "auto-cols": $()
      }],
      /**
       * Grid Auto Rows
       * @see https://tailwindcss.com/docs/grid-auto-rows
       */
      "auto-rows": [{
        "auto-rows": $()
      }],
      /**
       * Gap
       * @see https://tailwindcss.com/docs/gap
       */
      gap: [{
        gap: N()
      }],
      /**
       * Gap X
       * @see https://tailwindcss.com/docs/gap
       */
      "gap-x": [{
        "gap-x": N()
      }],
      /**
       * Gap Y
       * @see https://tailwindcss.com/docs/gap
       */
      "gap-y": [{
        "gap-y": N()
      }],
      /**
       * Justify Content
       * @see https://tailwindcss.com/docs/justify-content
       */
      "justify-content": [{
        justify: [...H(), "normal"]
      }],
      /**
       * Justify Items
       * @see https://tailwindcss.com/docs/justify-items
       */
      "justify-items": [{
        "justify-items": [...U(), "normal"]
      }],
      /**
       * Justify Self
       * @see https://tailwindcss.com/docs/justify-self
       */
      "justify-self": [{
        "justify-self": ["auto", ...U()]
      }],
      /**
       * Align Content
       * @see https://tailwindcss.com/docs/align-content
       */
      "align-content": [{
        content: ["normal", ...H()]
      }],
      /**
       * Align Items
       * @see https://tailwindcss.com/docs/align-items
       */
      "align-items": [{
        items: [...U(), {
          baseline: ["", "last"]
        }]
      }],
      /**
       * Align Self
       * @see https://tailwindcss.com/docs/align-self
       */
      "align-self": [{
        self: ["auto", ...U(), {
          baseline: ["", "last"]
        }]
      }],
      /**
       * Place Content
       * @see https://tailwindcss.com/docs/place-content
       */
      "place-content": [{
        "place-content": H()
      }],
      /**
       * Place Items
       * @see https://tailwindcss.com/docs/place-items
       */
      "place-items": [{
        "place-items": [...U(), "baseline"]
      }],
      /**
       * Place Self
       * @see https://tailwindcss.com/docs/place-self
       */
      "place-self": [{
        "place-self": ["auto", ...U()]
      }],
      // Spacing
      /**
       * Padding
       * @see https://tailwindcss.com/docs/padding
       */
      p: [{
        p: N()
      }],
      /**
       * Padding Inline
       * @see https://tailwindcss.com/docs/padding
       */
      px: [{
        px: N()
      }],
      /**
       * Padding Block
       * @see https://tailwindcss.com/docs/padding
       */
      py: [{
        py: N()
      }],
      /**
       * Padding Inline Start
       * @see https://tailwindcss.com/docs/padding
       */
      ps: [{
        ps: N()
      }],
      /**
       * Padding Inline End
       * @see https://tailwindcss.com/docs/padding
       */
      pe: [{
        pe: N()
      }],
      /**
       * Padding Block Start
       * @see https://tailwindcss.com/docs/padding
       */
      pbs: [{
        pbs: N()
      }],
      /**
       * Padding Block End
       * @see https://tailwindcss.com/docs/padding
       */
      pbe: [{
        pbe: N()
      }],
      /**
       * Padding Top
       * @see https://tailwindcss.com/docs/padding
       */
      pt: [{
        pt: N()
      }],
      /**
       * Padding Right
       * @see https://tailwindcss.com/docs/padding
       */
      pr: [{
        pr: N()
      }],
      /**
       * Padding Bottom
       * @see https://tailwindcss.com/docs/padding
       */
      pb: [{
        pb: N()
      }],
      /**
       * Padding Left
       * @see https://tailwindcss.com/docs/padding
       */
      pl: [{
        pl: N()
      }],
      /**
       * Margin
       * @see https://tailwindcss.com/docs/margin
       */
      m: [{
        m: O()
      }],
      /**
       * Margin Inline
       * @see https://tailwindcss.com/docs/margin
       */
      mx: [{
        mx: O()
      }],
      /**
       * Margin Block
       * @see https://tailwindcss.com/docs/margin
       */
      my: [{
        my: O()
      }],
      /**
       * Margin Inline Start
       * @see https://tailwindcss.com/docs/margin
       */
      ms: [{
        ms: O()
      }],
      /**
       * Margin Inline End
       * @see https://tailwindcss.com/docs/margin
       */
      me: [{
        me: O()
      }],
      /**
       * Margin Block Start
       * @see https://tailwindcss.com/docs/margin
       */
      mbs: [{
        mbs: O()
      }],
      /**
       * Margin Block End
       * @see https://tailwindcss.com/docs/margin
       */
      mbe: [{
        mbe: O()
      }],
      /**
       * Margin Top
       * @see https://tailwindcss.com/docs/margin
       */
      mt: [{
        mt: O()
      }],
      /**
       * Margin Right
       * @see https://tailwindcss.com/docs/margin
       */
      mr: [{
        mr: O()
      }],
      /**
       * Margin Bottom
       * @see https://tailwindcss.com/docs/margin
       */
      mb: [{
        mb: O()
      }],
      /**
       * Margin Left
       * @see https://tailwindcss.com/docs/margin
       */
      ml: [{
        ml: O()
      }],
      /**
       * Space Between X
       * @see https://tailwindcss.com/docs/margin#adding-space-between-children
       */
      "space-x": [{
        "space-x": N()
      }],
      /**
       * Space Between X Reverse
       * @see https://tailwindcss.com/docs/margin#adding-space-between-children
       */
      "space-x-reverse": ["space-x-reverse"],
      /**
       * Space Between Y
       * @see https://tailwindcss.com/docs/margin#adding-space-between-children
       */
      "space-y": [{
        "space-y": N()
      }],
      /**
       * Space Between Y Reverse
       * @see https://tailwindcss.com/docs/margin#adding-space-between-children
       */
      "space-y-reverse": ["space-y-reverse"],
      // --------------
      // --- Sizing ---
      // --------------
      /**
       * Size
       * @see https://tailwindcss.com/docs/width#setting-both-width-and-height
       */
      size: [{
        size: G()
      }],
      /**
       * Inline Size
       * @see https://tailwindcss.com/docs/width
       */
      "inline-size": [{
        inline: ["auto", ...M()]
      }],
      /**
       * Min-Inline Size
       * @see https://tailwindcss.com/docs/min-width
       */
      "min-inline-size": [{
        "min-inline": ["auto", ...M()]
      }],
      /**
       * Max-Inline Size
       * @see https://tailwindcss.com/docs/max-width
       */
      "max-inline-size": [{
        "max-inline": ["none", ...M()]
      }],
      /**
       * Block Size
       * @see https://tailwindcss.com/docs/height
       */
      "block-size": [{
        block: ["auto", ...K()]
      }],
      /**
       * Min-Block Size
       * @see https://tailwindcss.com/docs/min-height
       */
      "min-block-size": [{
        "min-block": ["auto", ...K()]
      }],
      /**
       * Max-Block Size
       * @see https://tailwindcss.com/docs/max-height
       */
      "max-block-size": [{
        "max-block": ["none", ...K()]
      }],
      /**
       * Width
       * @see https://tailwindcss.com/docs/width
       */
      w: [{
        w: [a, "screen", ...G()]
      }],
      /**
       * Min-Width
       * @see https://tailwindcss.com/docs/min-width
       */
      "min-w": [{
        "min-w": [
          a,
          "screen",
          /** Deprecated. @see https://github.com/tailwindlabs/tailwindcss.com/issues/2027#issuecomment-2620152757 */
          "none",
          ...G()
        ]
      }],
      /**
       * Max-Width
       * @see https://tailwindcss.com/docs/max-width
       */
      "max-w": [{
        "max-w": [
          a,
          "screen",
          "none",
          /** Deprecated since Tailwind CSS v4.0.0. @see https://github.com/tailwindlabs/tailwindcss.com/issues/2027#issuecomment-2620152757 */
          "prose",
          /** Deprecated since Tailwind CSS v4.0.0. @see https://github.com/tailwindlabs/tailwindcss.com/issues/2027#issuecomment-2620152757 */
          {
            screen: [i]
          },
          ...G()
        ]
      }],
      /**
       * Height
       * @see https://tailwindcss.com/docs/height
       */
      h: [{
        h: ["screen", "lh", ...G()]
      }],
      /**
       * Min-Height
       * @see https://tailwindcss.com/docs/min-height
       */
      "min-h": [{
        "min-h": ["screen", "lh", "none", ...G()]
      }],
      /**
       * Max-Height
       * @see https://tailwindcss.com/docs/max-height
       */
      "max-h": [{
        "max-h": ["screen", "lh", ...G()]
      }],
      // ------------------
      // --- Typography ---
      // ------------------
      /**
       * Font Size
       * @see https://tailwindcss.com/docs/font-size
       */
      "font-size": [{
        text: ["base", n, at, We]
      }],
      /**
       * Font Smoothing
       * @see https://tailwindcss.com/docs/font-smoothing
       */
      "font-smoothing": ["antialiased", "subpixel-antialiased"],
      /**
       * Font Style
       * @see https://tailwindcss.com/docs/font-style
       */
      "font-style": ["italic", "not-italic"],
      /**
       * Font Weight
       * @see https://tailwindcss.com/docs/font-weight
       */
      "font-weight": [{
        font: [r, Na, ba]
      }],
      /**
       * Font Stretch
       * @see https://tailwindcss.com/docs/font-stretch
       */
      "font-stretch": [{
        "font-stretch": ["ultra-condensed", "extra-condensed", "condensed", "semi-condensed", "normal", "semi-expanded", "expanded", "extra-expanded", "ultra-expanded", nn, A]
      }],
      /**
       * Font Family
       * @see https://tailwindcss.com/docs/font-family
       */
      "font-family": [{
        font: [xa, ya, t]
      }],
      /**
       * Font Feature Settings
       * @see https://tailwindcss.com/docs/font-feature-settings
       */
      "font-features": [{
        "font-features": [A]
      }],
      /**
       * Font Variant Numeric
       * @see https://tailwindcss.com/docs/font-variant-numeric
       */
      "fvn-normal": ["normal-nums"],
      /**
       * Font Variant Numeric
       * @see https://tailwindcss.com/docs/font-variant-numeric
       */
      "fvn-ordinal": ["ordinal"],
      /**
       * Font Variant Numeric
       * @see https://tailwindcss.com/docs/font-variant-numeric
       */
      "fvn-slashed-zero": ["slashed-zero"],
      /**
       * Font Variant Numeric
       * @see https://tailwindcss.com/docs/font-variant-numeric
       */
      "fvn-figure": ["lining-nums", "oldstyle-nums"],
      /**
       * Font Variant Numeric
       * @see https://tailwindcss.com/docs/font-variant-numeric
       */
      "fvn-spacing": ["proportional-nums", "tabular-nums"],
      /**
       * Font Variant Numeric
       * @see https://tailwindcss.com/docs/font-variant-numeric
       */
      "fvn-fraction": ["diagonal-fractions", "stacked-fractions"],
      /**
       * Letter Spacing
       * @see https://tailwindcss.com/docs/letter-spacing
       */
      tracking: [{
        tracking: [o, T, A]
      }],
      /**
       * Line Clamp
       * @see https://tailwindcss.com/docs/line-clamp
       */
      "line-clamp": [{
        "line-clamp": [z, "none", T, nr]
      }],
      /**
       * Line Height
       * @see https://tailwindcss.com/docs/line-height
       */
      leading: [{
        leading: [
          /** Deprecated since Tailwind CSS v4.0.0. @see https://github.com/tailwindlabs/tailwindcss.com/issues/2027#issuecomment-2620152757 */
          s,
          ...N()
        ]
      }],
      /**
       * List Style Image
       * @see https://tailwindcss.com/docs/list-style-image
       */
      "list-image": [{
        "list-image": ["none", T, A]
      }],
      /**
       * List Style Position
       * @see https://tailwindcss.com/docs/list-style-position
       */
      "list-style-position": [{
        list: ["inside", "outside"]
      }],
      /**
       * List Style Type
       * @see https://tailwindcss.com/docs/list-style-type
       */
      "list-style-type": [{
        list: ["disc", "decimal", "none", T, A]
      }],
      /**
       * Text Alignment
       * @see https://tailwindcss.com/docs/text-align
       */
      "text-alignment": [{
        text: ["left", "center", "right", "justify", "start", "end"]
      }],
      /**
       * Placeholder Color
       * @deprecated since Tailwind CSS v3.0.0
       * @see https://v3.tailwindcss.com/docs/placeholder-color
       */
      "placeholder-color": [{
        placeholder: P()
      }],
      /**
       * Text Color
       * @see https://tailwindcss.com/docs/text-color
       */
      "text-color": [{
        text: P()
      }],
      /**
       * Text Decoration
       * @see https://tailwindcss.com/docs/text-decoration
       */
      "text-decoration": ["underline", "overline", "line-through", "no-underline"],
      /**
       * Text Decoration Style
       * @see https://tailwindcss.com/docs/text-decoration-style
       */
      "text-decoration-style": [{
        decoration: [...D(), "wavy"]
      }],
      /**
       * Text Decoration Thickness
       * @see https://tailwindcss.com/docs/text-decoration-thickness
       */
      "text-decoration-thickness": [{
        decoration: [z, "from-font", "auto", T, We]
      }],
      /**
       * Text Decoration Color
       * @see https://tailwindcss.com/docs/text-decoration-color
       */
      "text-decoration-color": [{
        decoration: P()
      }],
      /**
       * Text Underline Offset
       * @see https://tailwindcss.com/docs/text-underline-offset
       */
      "underline-offset": [{
        "underline-offset": [z, "auto", T, A]
      }],
      /**
       * Text Transform
       * @see https://tailwindcss.com/docs/text-transform
       */
      "text-transform": ["uppercase", "lowercase", "capitalize", "normal-case"],
      /**
       * Text Overflow
       * @see https://tailwindcss.com/docs/text-overflow
       */
      "text-overflow": ["truncate", "text-ellipsis", "text-clip"],
      /**
       * Text Wrap
       * @see https://tailwindcss.com/docs/text-wrap
       */
      "text-wrap": [{
        text: ["wrap", "nowrap", "balance", "pretty"]
      }],
      /**
       * Text Indent
       * @see https://tailwindcss.com/docs/text-indent
       */
      indent: [{
        indent: N()
      }],
      /**
       * Tab Size
       * @see https://tailwindcss.com/docs/tab-size
       */
      "tab-size": [{
        tab: [ve, T, A]
      }],
      /**
       * Vertical Alignment
       * @see https://tailwindcss.com/docs/vertical-align
       */
      "vertical-align": [{
        align: ["baseline", "top", "middle", "bottom", "text-top", "text-bottom", "sub", "super", T, A]
      }],
      /**
       * Whitespace
       * @see https://tailwindcss.com/docs/whitespace
       */
      whitespace: [{
        whitespace: ["normal", "nowrap", "pre", "pre-line", "pre-wrap", "break-spaces"]
      }],
      /**
       * Word Break
       * @see https://tailwindcss.com/docs/word-break
       */
      break: [{
        break: ["normal", "words", "all", "keep"]
      }],
      /**
       * Overflow Wrap
       * @see https://tailwindcss.com/docs/overflow-wrap
       */
      wrap: [{
        wrap: ["break-word", "anywhere", "normal"]
      }],
      /**
       * Hyphens
       * @see https://tailwindcss.com/docs/hyphens
       */
      hyphens: [{
        hyphens: ["none", "manual", "auto"]
      }],
      /**
       * Content
       * @see https://tailwindcss.com/docs/content
       */
      content: [{
        content: ["none", T, A]
      }],
      // -------------------
      // --- Backgrounds ---
      // -------------------
      /**
       * Background Attachment
       * @see https://tailwindcss.com/docs/background-attachment
       */
      "bg-attachment": [{
        bg: ["fixed", "local", "scroll"]
      }],
      /**
       * Background Clip
       * @see https://tailwindcss.com/docs/background-clip
       */
      "bg-clip": [{
        "bg-clip": ["border", "padding", "content", "text"]
      }],
      /**
       * Background Origin
       * @see https://tailwindcss.com/docs/background-origin
       */
      "bg-origin": [{
        "bg-origin": ["border", "padding", "content"]
      }],
      /**
       * Background Position
       * @see https://tailwindcss.com/docs/background-position
       */
      "bg-position": [{
        bg: me()
      }],
      /**
       * Background Repeat
       * @see https://tailwindcss.com/docs/background-repeat
       */
      "bg-repeat": [{
        bg: ee()
      }],
      /**
       * Background Size
       * @see https://tailwindcss.com/docs/background-size
       */
      "bg-size": [{
        bg: ce()
      }],
      /**
       * Background Image
       * @see https://tailwindcss.com/docs/background-image
       */
      "bg-image": [{
        bg: ["none", {
          linear: [{
            to: ["t", "tr", "r", "br", "b", "bl", "l", "tl"]
          }, ve, T, A],
          radial: ["", T, A],
          conic: [ve, T, A]
        }, Sa, wa]
      }],
      /**
       * Background Color
       * @see https://tailwindcss.com/docs/background-color
       */
      "bg-color": [{
        bg: P()
      }],
      /**
       * Gradient Color Stops From Position
       * @see https://tailwindcss.com/docs/gradient-color-stops
       */
      "gradient-from-pos": [{
        from: le()
      }],
      /**
       * Gradient Color Stops Via Position
       * @see https://tailwindcss.com/docs/gradient-color-stops
       */
      "gradient-via-pos": [{
        via: le()
      }],
      /**
       * Gradient Color Stops To Position
       * @see https://tailwindcss.com/docs/gradient-color-stops
       */
      "gradient-to-pos": [{
        to: le()
      }],
      /**
       * Gradient Color Stops From
       * @see https://tailwindcss.com/docs/gradient-color-stops
       */
      "gradient-from": [{
        from: P()
      }],
      /**
       * Gradient Color Stops Via
       * @see https://tailwindcss.com/docs/gradient-color-stops
       */
      "gradient-via": [{
        via: P()
      }],
      /**
       * Gradient Color Stops To
       * @see https://tailwindcss.com/docs/gradient-color-stops
       */
      "gradient-to": [{
        to: P()
      }],
      // ---------------
      // --- Borders ---
      // ---------------
      /**
       * Border Radius
       * @see https://tailwindcss.com/docs/border-radius
       */
      rounded: [{
        rounded: J()
      }],
      /**
       * Border Radius Start
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-s": [{
        "rounded-s": J()
      }],
      /**
       * Border Radius End
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-e": [{
        "rounded-e": J()
      }],
      /**
       * Border Radius Top
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-t": [{
        "rounded-t": J()
      }],
      /**
       * Border Radius Right
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-r": [{
        "rounded-r": J()
      }],
      /**
       * Border Radius Bottom
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-b": [{
        "rounded-b": J()
      }],
      /**
       * Border Radius Left
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-l": [{
        "rounded-l": J()
      }],
      /**
       * Border Radius Start Start
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-ss": [{
        "rounded-ss": J()
      }],
      /**
       * Border Radius Start End
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-se": [{
        "rounded-se": J()
      }],
      /**
       * Border Radius End End
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-ee": [{
        "rounded-ee": J()
      }],
      /**
       * Border Radius End Start
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-es": [{
        "rounded-es": J()
      }],
      /**
       * Border Radius Top Left
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-tl": [{
        "rounded-tl": J()
      }],
      /**
       * Border Radius Top Right
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-tr": [{
        "rounded-tr": J()
      }],
      /**
       * Border Radius Bottom Right
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-br": [{
        "rounded-br": J()
      }],
      /**
       * Border Radius Bottom Left
       * @see https://tailwindcss.com/docs/border-radius
       */
      "rounded-bl": [{
        "rounded-bl": J()
      }],
      /**
       * Border Width
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w": [{
        border: Q()
      }],
      /**
       * Border Width Inline
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-x": [{
        "border-x": Q()
      }],
      /**
       * Border Width Block
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-y": [{
        "border-y": Q()
      }],
      /**
       * Border Width Inline Start
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-s": [{
        "border-s": Q()
      }],
      /**
       * Border Width Inline End
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-e": [{
        "border-e": Q()
      }],
      /**
       * Border Width Block Start
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-bs": [{
        "border-bs": Q()
      }],
      /**
       * Border Width Block End
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-be": [{
        "border-be": Q()
      }],
      /**
       * Border Width Top
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-t": [{
        "border-t": Q()
      }],
      /**
       * Border Width Right
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-r": [{
        "border-r": Q()
      }],
      /**
       * Border Width Bottom
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-b": [{
        "border-b": Q()
      }],
      /**
       * Border Width Left
       * @see https://tailwindcss.com/docs/border-width
       */
      "border-w-l": [{
        "border-l": Q()
      }],
      /**
       * Divide Width X
       * @see https://tailwindcss.com/docs/border-width#between-children
       */
      "divide-x": [{
        "divide-x": Q()
      }],
      /**
       * Divide Width X Reverse
       * @see https://tailwindcss.com/docs/border-width#between-children
       */
      "divide-x-reverse": ["divide-x-reverse"],
      /**
       * Divide Width Y
       * @see https://tailwindcss.com/docs/border-width#between-children
       */
      "divide-y": [{
        "divide-y": Q()
      }],
      /**
       * Divide Width Y Reverse
       * @see https://tailwindcss.com/docs/border-width#between-children
       */
      "divide-y-reverse": ["divide-y-reverse"],
      /**
       * Border Style
       * @see https://tailwindcss.com/docs/border-style
       */
      "border-style": [{
        border: [...D(), "hidden", "none"]
      }],
      /**
       * Divide Style
       * @see https://tailwindcss.com/docs/border-style#setting-the-divider-style
       */
      "divide-style": [{
        divide: [...D(), "hidden", "none"]
      }],
      /**
       * Border Color
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color": [{
        border: P()
      }],
      /**
       * Border Color Inline
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-x": [{
        "border-x": P()
      }],
      /**
       * Border Color Block
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-y": [{
        "border-y": P()
      }],
      /**
       * Border Color Inline Start
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-s": [{
        "border-s": P()
      }],
      /**
       * Border Color Inline End
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-e": [{
        "border-e": P()
      }],
      /**
       * Border Color Block Start
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-bs": [{
        "border-bs": P()
      }],
      /**
       * Border Color Block End
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-be": [{
        "border-be": P()
      }],
      /**
       * Border Color Top
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-t": [{
        "border-t": P()
      }],
      /**
       * Border Color Right
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-r": [{
        "border-r": P()
      }],
      /**
       * Border Color Bottom
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-b": [{
        "border-b": P()
      }],
      /**
       * Border Color Left
       * @see https://tailwindcss.com/docs/border-color
       */
      "border-color-l": [{
        "border-l": P()
      }],
      /**
       * Divide Color
       * @see https://tailwindcss.com/docs/divide-color
       */
      "divide-color": [{
        divide: P()
      }],
      /**
       * Outline Style
       * @see https://tailwindcss.com/docs/outline-style
       */
      "outline-style": [{
        outline: [...D(), "none", "hidden"]
      }],
      /**
       * Outline Offset
       * @see https://tailwindcss.com/docs/outline-offset
       */
      "outline-offset": [{
        "outline-offset": [z, T, A]
      }],
      /**
       * Outline Width
       * @see https://tailwindcss.com/docs/outline-width
       */
      "outline-w": [{
        outline: ["", z, at, We]
      }],
      /**
       * Outline Color
       * @see https://tailwindcss.com/docs/outline-color
       */
      "outline-color": [{
        outline: P()
      }],
      // ---------------
      // --- Effects ---
      // ---------------
      /**
       * Box Shadow
       * @see https://tailwindcss.com/docs/box-shadow
       */
      shadow: [{
        shadow: [
          // Deprecated since Tailwind CSS v4.0.0
          "",
          "none",
          f,
          Ct,
          xt
        ]
      }],
      /**
       * Box Shadow Color
       * @see https://tailwindcss.com/docs/box-shadow#setting-the-shadow-color
       */
      "shadow-color": [{
        shadow: P()
      }],
      /**
       * Inset Box Shadow
       * @see https://tailwindcss.com/docs/box-shadow#adding-an-inset-shadow
       */
      "inset-shadow": [{
        "inset-shadow": ["none", m, Ct, xt]
      }],
      /**
       * Inset Box Shadow Color
       * @see https://tailwindcss.com/docs/box-shadow#setting-the-inset-shadow-color
       */
      "inset-shadow-color": [{
        "inset-shadow": P()
      }],
      /**
       * Ring Width
       * @see https://tailwindcss.com/docs/box-shadow#adding-a-ring
       */
      "ring-w": [{
        ring: Q()
      }],
      /**
       * Ring Width Inset
       * @see https://v3.tailwindcss.com/docs/ring-width#inset-rings
       * @deprecated since Tailwind CSS v4.0.0
       * @see https://github.com/tailwindlabs/tailwindcss/blob/v4.0.0/packages/tailwindcss/src/utilities.ts#L4158
       */
      "ring-w-inset": ["ring-inset"],
      /**
       * Ring Color
       * @see https://tailwindcss.com/docs/box-shadow#setting-the-ring-color
       */
      "ring-color": [{
        ring: P()
      }],
      /**
       * Ring Offset Width
       * @see https://v3.tailwindcss.com/docs/ring-offset-width
       * @deprecated since Tailwind CSS v4.0.0
       * @see https://github.com/tailwindlabs/tailwindcss/blob/v4.0.0/packages/tailwindcss/src/utilities.ts#L4158
       */
      "ring-offset-w": [{
        "ring-offset": [z, We]
      }],
      /**
       * Ring Offset Color
       * @see https://v3.tailwindcss.com/docs/ring-offset-color
       * @deprecated since Tailwind CSS v4.0.0
       * @see https://github.com/tailwindlabs/tailwindcss/blob/v4.0.0/packages/tailwindcss/src/utilities.ts#L4158
       */
      "ring-offset-color": [{
        "ring-offset": P()
      }],
      /**
       * Inset Ring Width
       * @see https://tailwindcss.com/docs/box-shadow#adding-an-inset-ring
       */
      "inset-ring-w": [{
        "inset-ring": Q()
      }],
      /**
       * Inset Ring Color
       * @see https://tailwindcss.com/docs/box-shadow#setting-the-inset-ring-color
       */
      "inset-ring-color": [{
        "inset-ring": P()
      }],
      /**
       * Text Shadow
       * @see https://tailwindcss.com/docs/text-shadow
       */
      "text-shadow": [{
        "text-shadow": ["none", h, Ct, xt]
      }],
      /**
       * Text Shadow Color
       * @see https://tailwindcss.com/docs/text-shadow#setting-the-shadow-color
       */
      "text-shadow-color": [{
        "text-shadow": P()
      }],
      /**
       * Opacity
       * @see https://tailwindcss.com/docs/opacity
       */
      opacity: [{
        opacity: [z, T, A]
      }],
      /**
       * Mix Blend Mode
       * @see https://tailwindcss.com/docs/mix-blend-mode
       */
      "mix-blend": [{
        "mix-blend": [...Z(), "plus-darker", "plus-lighter"]
      }],
      /**
       * Background Blend Mode
       * @see https://tailwindcss.com/docs/background-blend-mode
       */
      "bg-blend": [{
        "bg-blend": Z()
      }],
      /**
       * Mask Clip
       * @see https://tailwindcss.com/docs/mask-clip
       */
      "mask-clip": [{
        "mask-clip": ["border", "padding", "content", "fill", "stroke", "view"]
      }, "mask-no-clip"],
      /**
       * Mask Composite
       * @see https://tailwindcss.com/docs/mask-composite
       */
      "mask-composite": [{
        mask: ["add", "subtract", "intersect", "exclude"]
      }],
      /**
       * Mask Image
       * @see https://tailwindcss.com/docs/mask-image
       */
      "mask-image-linear-pos": [{
        "mask-linear": [z]
      }],
      "mask-image-linear-from-pos": [{
        "mask-linear-from": V()
      }],
      "mask-image-linear-to-pos": [{
        "mask-linear-to": V()
      }],
      "mask-image-linear-from-color": [{
        "mask-linear-from": P()
      }],
      "mask-image-linear-to-color": [{
        "mask-linear-to": P()
      }],
      "mask-image-t-from-pos": [{
        "mask-t-from": V()
      }],
      "mask-image-t-to-pos": [{
        "mask-t-to": V()
      }],
      "mask-image-t-from-color": [{
        "mask-t-from": P()
      }],
      "mask-image-t-to-color": [{
        "mask-t-to": P()
      }],
      "mask-image-r-from-pos": [{
        "mask-r-from": V()
      }],
      "mask-image-r-to-pos": [{
        "mask-r-to": V()
      }],
      "mask-image-r-from-color": [{
        "mask-r-from": P()
      }],
      "mask-image-r-to-color": [{
        "mask-r-to": P()
      }],
      "mask-image-b-from-pos": [{
        "mask-b-from": V()
      }],
      "mask-image-b-to-pos": [{
        "mask-b-to": V()
      }],
      "mask-image-b-from-color": [{
        "mask-b-from": P()
      }],
      "mask-image-b-to-color": [{
        "mask-b-to": P()
      }],
      "mask-image-l-from-pos": [{
        "mask-l-from": V()
      }],
      "mask-image-l-to-pos": [{
        "mask-l-to": V()
      }],
      "mask-image-l-from-color": [{
        "mask-l-from": P()
      }],
      "mask-image-l-to-color": [{
        "mask-l-to": P()
      }],
      "mask-image-x-from-pos": [{
        "mask-x-from": V()
      }],
      "mask-image-x-to-pos": [{
        "mask-x-to": V()
      }],
      "mask-image-x-from-color": [{
        "mask-x-from": P()
      }],
      "mask-image-x-to-color": [{
        "mask-x-to": P()
      }],
      "mask-image-y-from-pos": [{
        "mask-y-from": V()
      }],
      "mask-image-y-to-pos": [{
        "mask-y-to": V()
      }],
      "mask-image-y-from-color": [{
        "mask-y-from": P()
      }],
      "mask-image-y-to-color": [{
        "mask-y-to": P()
      }],
      "mask-image-radial": [{
        "mask-radial": [T, A]
      }],
      "mask-image-radial-from-pos": [{
        "mask-radial-from": V()
      }],
      "mask-image-radial-to-pos": [{
        "mask-radial-to": V()
      }],
      "mask-image-radial-from-color": [{
        "mask-radial-from": P()
      }],
      "mask-image-radial-to-color": [{
        "mask-radial-to": P()
      }],
      "mask-image-radial-shape": [{
        "mask-radial": ["circle", "ellipse"]
      }],
      "mask-image-radial-size": [{
        "mask-radial": [{
          closest: ["side", "corner"],
          farthest: ["side", "corner"]
        }]
      }],
      "mask-image-radial-pos": [{
        "mask-radial-at": S()
      }],
      "mask-image-conic-pos": [{
        "mask-conic": [z]
      }],
      "mask-image-conic-from-pos": [{
        "mask-conic-from": V()
      }],
      "mask-image-conic-to-pos": [{
        "mask-conic-to": V()
      }],
      "mask-image-conic-from-color": [{
        "mask-conic-from": P()
      }],
      "mask-image-conic-to-color": [{
        "mask-conic-to": P()
      }],
      /**
       * Mask Mode
       * @see https://tailwindcss.com/docs/mask-mode
       */
      "mask-mode": [{
        mask: ["alpha", "luminance", "match"]
      }],
      /**
       * Mask Origin
       * @see https://tailwindcss.com/docs/mask-origin
       */
      "mask-origin": [{
        "mask-origin": ["border", "padding", "content", "fill", "stroke", "view"]
      }],
      /**
       * Mask Position
       * @see https://tailwindcss.com/docs/mask-position
       */
      "mask-position": [{
        mask: me()
      }],
      /**
       * Mask Repeat
       * @see https://tailwindcss.com/docs/mask-repeat
       */
      "mask-repeat": [{
        mask: ee()
      }],
      /**
       * Mask Size
       * @see https://tailwindcss.com/docs/mask-size
       */
      "mask-size": [{
        mask: ce()
      }],
      /**
       * Mask Type
       * @see https://tailwindcss.com/docs/mask-type
       */
      "mask-type": [{
        "mask-type": ["alpha", "luminance"]
      }],
      /**
       * Mask Image
       * @see https://tailwindcss.com/docs/mask-image
       */
      "mask-image": [{
        mask: ["none", T, A]
      }],
      // ---------------
      // --- Filters ---
      // ---------------
      /**
       * Filter
       * @see https://tailwindcss.com/docs/filter
       */
      filter: [{
        filter: [
          // Deprecated since Tailwind CSS v3.0.0
          "",
          "none",
          T,
          A
        ]
      }],
      /**
       * Blur
       * @see https://tailwindcss.com/docs/blur
       */
      blur: [{
        blur: X()
      }],
      /**
       * Brightness
       * @see https://tailwindcss.com/docs/brightness
       */
      brightness: [{
        brightness: [z, T, A]
      }],
      /**
       * Contrast
       * @see https://tailwindcss.com/docs/contrast
       */
      contrast: [{
        contrast: [z, T, A]
      }],
      /**
       * Drop Shadow
       * @see https://tailwindcss.com/docs/drop-shadow
       */
      "drop-shadow": [{
        "drop-shadow": [
          // Deprecated since Tailwind CSS v4.0.0
          "",
          "none",
          g,
          Ct,
          xt
        ]
      }],
      /**
       * Drop Shadow Color
       * @see https://tailwindcss.com/docs/filter-drop-shadow#setting-the-shadow-color
       */
      "drop-shadow-color": [{
        "drop-shadow": P()
      }],
      /**
       * Grayscale
       * @see https://tailwindcss.com/docs/grayscale
       */
      grayscale: [{
        grayscale: ["", z, T, A]
      }],
      /**
       * Hue Rotate
       * @see https://tailwindcss.com/docs/hue-rotate
       */
      "hue-rotate": [{
        "hue-rotate": [z, T, A]
      }],
      /**
       * Invert
       * @see https://tailwindcss.com/docs/invert
       */
      invert: [{
        invert: ["", z, T, A]
      }],
      /**
       * Saturate
       * @see https://tailwindcss.com/docs/saturate
       */
      saturate: [{
        saturate: [z, T, A]
      }],
      /**
       * Sepia
       * @see https://tailwindcss.com/docs/sepia
       */
      sepia: [{
        sepia: ["", z, T, A]
      }],
      /**
       * Backdrop Filter
       * @see https://tailwindcss.com/docs/backdrop-filter
       */
      "backdrop-filter": [{
        "backdrop-filter": [
          // Deprecated since Tailwind CSS v3.0.0
          "",
          "none",
          T,
          A
        ]
      }],
      /**
       * Backdrop Blur
       * @see https://tailwindcss.com/docs/backdrop-blur
       */
      "backdrop-blur": [{
        "backdrop-blur": X()
      }],
      /**
       * Backdrop Brightness
       * @see https://tailwindcss.com/docs/backdrop-brightness
       */
      "backdrop-brightness": [{
        "backdrop-brightness": [z, T, A]
      }],
      /**
       * Backdrop Contrast
       * @see https://tailwindcss.com/docs/backdrop-contrast
       */
      "backdrop-contrast": [{
        "backdrop-contrast": [z, T, A]
      }],
      /**
       * Backdrop Grayscale
       * @see https://tailwindcss.com/docs/backdrop-grayscale
       */
      "backdrop-grayscale": [{
        "backdrop-grayscale": ["", z, T, A]
      }],
      /**
       * Backdrop Hue Rotate
       * @see https://tailwindcss.com/docs/backdrop-hue-rotate
       */
      "backdrop-hue-rotate": [{
        "backdrop-hue-rotate": [z, T, A]
      }],
      /**
       * Backdrop Invert
       * @see https://tailwindcss.com/docs/backdrop-invert
       */
      "backdrop-invert": [{
        "backdrop-invert": ["", z, T, A]
      }],
      /**
       * Backdrop Opacity
       * @see https://tailwindcss.com/docs/backdrop-opacity
       */
      "backdrop-opacity": [{
        "backdrop-opacity": [z, T, A]
      }],
      /**
       * Backdrop Saturate
       * @see https://tailwindcss.com/docs/backdrop-saturate
       */
      "backdrop-saturate": [{
        "backdrop-saturate": [z, T, A]
      }],
      /**
       * Backdrop Sepia
       * @see https://tailwindcss.com/docs/backdrop-sepia
       */
      "backdrop-sepia": [{
        "backdrop-sepia": ["", z, T, A]
      }],
      // --------------
      // --- Tables ---
      // --------------
      /**
       * Border Collapse
       * @see https://tailwindcss.com/docs/border-collapse
       */
      "border-collapse": [{
        border: ["collapse", "separate"]
      }],
      /**
       * Border Spacing
       * @see https://tailwindcss.com/docs/border-spacing
       */
      "border-spacing": [{
        "border-spacing": N()
      }],
      /**
       * Border Spacing X
       * @see https://tailwindcss.com/docs/border-spacing
       */
      "border-spacing-x": [{
        "border-spacing-x": N()
      }],
      /**
       * Border Spacing Y
       * @see https://tailwindcss.com/docs/border-spacing
       */
      "border-spacing-y": [{
        "border-spacing-y": N()
      }],
      /**
       * Table Layout
       * @see https://tailwindcss.com/docs/table-layout
       */
      "table-layout": [{
        table: ["auto", "fixed"]
      }],
      /**
       * Caption Side
       * @see https://tailwindcss.com/docs/caption-side
       */
      caption: [{
        caption: ["top", "bottom"]
      }],
      // ---------------------------------
      // --- Transitions and Animation ---
      // ---------------------------------
      /**
       * Transition Property
       * @see https://tailwindcss.com/docs/transition-property
       */
      transition: [{
        transition: ["", "all", "colors", "opacity", "shadow", "transform", "none", T, A]
      }],
      /**
       * Transition Behavior
       * @see https://tailwindcss.com/docs/transition-behavior
       */
      "transition-behavior": [{
        transition: ["normal", "discrete"]
      }],
      /**
       * Transition Duration
       * @see https://tailwindcss.com/docs/transition-duration
       */
      duration: [{
        duration: [z, "initial", T, A]
      }],
      /**
       * Transition Timing Function
       * @see https://tailwindcss.com/docs/transition-timing-function
       */
      ease: [{
        ease: ["linear", "initial", b, T, A]
      }],
      /**
       * Transition Delay
       * @see https://tailwindcss.com/docs/transition-delay
       */
      delay: [{
        delay: [z, T, A]
      }],
      /**
       * Animation
       * @see https://tailwindcss.com/docs/animation
       */
      animate: [{
        animate: ["none", y, T, A]
      }],
      // ------------------
      // --- Transforms ---
      // ------------------
      /**
       * Backface Visibility
       * @see https://tailwindcss.com/docs/backface-visibility
       */
      backface: [{
        backface: ["hidden", "visible"]
      }],
      /**
       * Perspective
       * @see https://tailwindcss.com/docs/perspective
       */
      perspective: [{
        perspective: [p, T, A]
      }],
      /**
       * Perspective Origin
       * @see https://tailwindcss.com/docs/perspective-origin
       */
      "perspective-origin": [{
        "perspective-origin": E()
      }],
      /**
       * Rotate
       * @see https://tailwindcss.com/docs/rotate
       */
      rotate: [{
        rotate: W()
      }],
      /**
       * Rotate X
       * @see https://tailwindcss.com/docs/rotate
       */
      "rotate-x": [{
        "rotate-x": W()
      }],
      /**
       * Rotate Y
       * @see https://tailwindcss.com/docs/rotate
       */
      "rotate-y": [{
        "rotate-y": W()
      }],
      /**
       * Rotate Z
       * @see https://tailwindcss.com/docs/rotate
       */
      "rotate-z": [{
        "rotate-z": W()
      }],
      /**
       * Scale
       * @see https://tailwindcss.com/docs/scale
       */
      scale: [{
        scale: j()
      }],
      /**
       * Scale X
       * @see https://tailwindcss.com/docs/scale
       */
      "scale-x": [{
        "scale-x": j()
      }],
      /**
       * Scale Y
       * @see https://tailwindcss.com/docs/scale
       */
      "scale-y": [{
        "scale-y": j()
      }],
      /**
       * Scale Z
       * @see https://tailwindcss.com/docs/scale
       */
      "scale-z": [{
        "scale-z": j()
      }],
      /**
       * Scale 3D
       * @see https://tailwindcss.com/docs/scale
       */
      "scale-3d": ["scale-3d"],
      /**
       * Skew
       * @see https://tailwindcss.com/docs/skew
       */
      skew: [{
        skew: de()
      }],
      /**
       * Skew X
       * @see https://tailwindcss.com/docs/skew
       */
      "skew-x": [{
        "skew-x": de()
      }],
      /**
       * Skew Y
       * @see https://tailwindcss.com/docs/skew
       */
      "skew-y": [{
        "skew-y": de()
      }],
      /**
       * Transform
       * @see https://tailwindcss.com/docs/transform
       */
      transform: [{
        transform: [T, A, "", "none", "gpu", "cpu"]
      }],
      /**
       * Transform Origin
       * @see https://tailwindcss.com/docs/transform-origin
       */
      "transform-origin": [{
        origin: E()
      }],
      /**
       * Transform Style
       * @see https://tailwindcss.com/docs/transform-style
       */
      "transform-style": [{
        transform: ["3d", "flat"]
      }],
      /**
       * Translate
       * @see https://tailwindcss.com/docs/translate
       */
      translate: [{
        translate: ae()
      }],
      /**
       * Translate X
       * @see https://tailwindcss.com/docs/translate
       */
      "translate-x": [{
        "translate-x": ae()
      }],
      /**
       * Translate Y
       * @see https://tailwindcss.com/docs/translate
       */
      "translate-y": [{
        "translate-y": ae()
      }],
      /**
       * Translate Z
       * @see https://tailwindcss.com/docs/translate
       */
      "translate-z": [{
        "translate-z": ae()
      }],
      /**
       * Translate None
       * @see https://tailwindcss.com/docs/translate
       */
      "translate-none": ["translate-none"],
      /**
       * Zoom
       * @see https://tailwindcss.com/docs/zoom
       */
      zoom: [{
        zoom: [ve, T, A]
      }],
      // ---------------------
      // --- Interactivity ---
      // ---------------------
      /**
       * Accent Color
       * @see https://tailwindcss.com/docs/accent-color
       */
      accent: [{
        accent: P()
      }],
      /**
       * Appearance
       * @see https://tailwindcss.com/docs/appearance
       */
      appearance: [{
        appearance: ["none", "auto"]
      }],
      /**
       * Caret Color
       * @see https://tailwindcss.com/docs/just-in-time-mode#caret-color-utilities
       */
      "caret-color": [{
        caret: P()
      }],
      /**
       * Color Scheme
       * @see https://tailwindcss.com/docs/color-scheme
       */
      "color-scheme": [{
        scheme: ["normal", "dark", "light", "light-dark", "only-dark", "only-light"]
      }],
      /**
       * Cursor
       * @see https://tailwindcss.com/docs/cursor
       */
      cursor: [{
        cursor: ["auto", "default", "pointer", "wait", "text", "move", "help", "not-allowed", "none", "context-menu", "progress", "cell", "crosshair", "vertical-text", "alias", "copy", "no-drop", "grab", "grabbing", "all-scroll", "col-resize", "row-resize", "n-resize", "e-resize", "s-resize", "w-resize", "ne-resize", "nw-resize", "se-resize", "sw-resize", "ew-resize", "ns-resize", "nesw-resize", "nwse-resize", "zoom-in", "zoom-out", T, A]
      }],
      /**
       * Field Sizing
       * @see https://tailwindcss.com/docs/field-sizing
       */
      "field-sizing": [{
        "field-sizing": ["fixed", "content"]
      }],
      /**
       * Pointer Events
       * @see https://tailwindcss.com/docs/pointer-events
       */
      "pointer-events": [{
        "pointer-events": ["auto", "none"]
      }],
      /**
       * Resize
       * @see https://tailwindcss.com/docs/resize
       */
      resize: [{
        resize: ["none", "", "y", "x"]
      }],
      /**
       * Scroll Behavior
       * @see https://tailwindcss.com/docs/scroll-behavior
       */
      "scroll-behavior": [{
        scroll: ["auto", "smooth"]
      }],
      /**
       * Scrollbar Thumb Color
       * @see https://tailwindcss.com/docs/scrollbar-color
       */
      "scrollbar-thumb-color": [{
        "scrollbar-thumb": P()
      }],
      /**
       * Scrollbar Track Color
       * @see https://tailwindcss.com/docs/scrollbar-color
       */
      "scrollbar-track-color": [{
        "scrollbar-track": P()
      }],
      /**
       * Scrollbar Gutter
       * @see https://tailwindcss.com/docs/scrollbar-gutter
       */
      "scrollbar-gutter": [{
        "scrollbar-gutter": ["auto", "stable", "both"]
      }],
      /**
       * Scrollbar Width
       * @see https://tailwindcss.com/docs/scrollbar-width
       */
      "scrollbar-w": [{
        scrollbar: ["auto", "thin", "none"]
      }],
      /**
       * Scroll Margin
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-m": [{
        "scroll-m": N()
      }],
      /**
       * Scroll Margin Inline
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-mx": [{
        "scroll-mx": N()
      }],
      /**
       * Scroll Margin Block
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-my": [{
        "scroll-my": N()
      }],
      /**
       * Scroll Margin Inline Start
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-ms": [{
        "scroll-ms": N()
      }],
      /**
       * Scroll Margin Inline End
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-me": [{
        "scroll-me": N()
      }],
      /**
       * Scroll Margin Block Start
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-mbs": [{
        "scroll-mbs": N()
      }],
      /**
       * Scroll Margin Block End
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-mbe": [{
        "scroll-mbe": N()
      }],
      /**
       * Scroll Margin Top
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-mt": [{
        "scroll-mt": N()
      }],
      /**
       * Scroll Margin Right
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-mr": [{
        "scroll-mr": N()
      }],
      /**
       * Scroll Margin Bottom
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-mb": [{
        "scroll-mb": N()
      }],
      /**
       * Scroll Margin Left
       * @see https://tailwindcss.com/docs/scroll-margin
       */
      "scroll-ml": [{
        "scroll-ml": N()
      }],
      /**
       * Scroll Padding
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-p": [{
        "scroll-p": N()
      }],
      /**
       * Scroll Padding Inline
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-px": [{
        "scroll-px": N()
      }],
      /**
       * Scroll Padding Block
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-py": [{
        "scroll-py": N()
      }],
      /**
       * Scroll Padding Inline Start
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-ps": [{
        "scroll-ps": N()
      }],
      /**
       * Scroll Padding Inline End
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-pe": [{
        "scroll-pe": N()
      }],
      /**
       * Scroll Padding Block Start
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-pbs": [{
        "scroll-pbs": N()
      }],
      /**
       * Scroll Padding Block End
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-pbe": [{
        "scroll-pbe": N()
      }],
      /**
       * Scroll Padding Top
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-pt": [{
        "scroll-pt": N()
      }],
      /**
       * Scroll Padding Right
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-pr": [{
        "scroll-pr": N()
      }],
      /**
       * Scroll Padding Bottom
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-pb": [{
        "scroll-pb": N()
      }],
      /**
       * Scroll Padding Left
       * @see https://tailwindcss.com/docs/scroll-padding
       */
      "scroll-pl": [{
        "scroll-pl": N()
      }],
      /**
       * Scroll Snap Align
       * @see https://tailwindcss.com/docs/scroll-snap-align
       */
      "snap-align": [{
        snap: ["start", "end", "center", "align-none"]
      }],
      /**
       * Scroll Snap Stop
       * @see https://tailwindcss.com/docs/scroll-snap-stop
       */
      "snap-stop": [{
        snap: ["normal", "always"]
      }],
      /**
       * Scroll Snap Type
       * @see https://tailwindcss.com/docs/scroll-snap-type
       */
      "snap-type": [{
        snap: ["none", "x", "y", "both"]
      }],
      /**
       * Scroll Snap Type Strictness
       * @see https://tailwindcss.com/docs/scroll-snap-type
       */
      "snap-strictness": [{
        snap: ["mandatory", "proximity"]
      }],
      /**
       * Touch Action
       * @see https://tailwindcss.com/docs/touch-action
       */
      touch: [{
        touch: ["auto", "none", "manipulation"]
      }],
      /**
       * Touch Action X
       * @see https://tailwindcss.com/docs/touch-action
       */
      "touch-x": [{
        "touch-pan": ["x", "left", "right"]
      }],
      /**
       * Touch Action Y
       * @see https://tailwindcss.com/docs/touch-action
       */
      "touch-y": [{
        "touch-pan": ["y", "up", "down"]
      }],
      /**
       * Touch Action Pinch Zoom
       * @see https://tailwindcss.com/docs/touch-action
       */
      "touch-pz": ["touch-pinch-zoom"],
      /**
       * User Select
       * @see https://tailwindcss.com/docs/user-select
       */
      select: [{
        select: ["none", "text", "all", "auto"]
      }],
      /**
       * Will Change
       * @see https://tailwindcss.com/docs/will-change
       */
      "will-change": [{
        "will-change": ["auto", "scroll", "contents", "transform", T, A]
      }],
      // -----------
      // --- SVG ---
      // -----------
      /**
       * Fill
       * @see https://tailwindcss.com/docs/fill
       */
      fill: [{
        fill: ["none", ...P()]
      }],
      /**
       * Stroke Width
       * @see https://tailwindcss.com/docs/stroke-width
       */
      "stroke-w": [{
        stroke: [z, at, We, nr]
      }],
      /**
       * Stroke
       * @see https://tailwindcss.com/docs/stroke
       */
      stroke: [{
        stroke: ["none", ...P()]
      }],
      // ---------------------
      // --- Accessibility ---
      // ---------------------
      /**
       * Forced Color Adjust
       * @see https://tailwindcss.com/docs/forced-color-adjust
       */
      "forced-color-adjust": [{
        "forced-color-adjust": ["auto", "none"]
      }]
    },
    conflictingClassGroups: {
      "container-named": ["container-type"],
      overflow: ["overflow-x", "overflow-y"],
      overscroll: ["overscroll-x", "overscroll-y"],
      inset: ["inset-x", "inset-y", "inset-bs", "inset-be", "start", "end", "top", "right", "bottom", "left"],
      "inset-x": ["right", "left"],
      "inset-y": ["top", "bottom"],
      flex: ["basis", "grow", "shrink"],
      gap: ["gap-x", "gap-y"],
      p: ["px", "py", "ps", "pe", "pbs", "pbe", "pt", "pr", "pb", "pl"],
      px: ["pr", "pl"],
      py: ["pt", "pb"],
      m: ["mx", "my", "ms", "me", "mbs", "mbe", "mt", "mr", "mb", "ml"],
      mx: ["mr", "ml"],
      my: ["mt", "mb"],
      size: ["w", "h"],
      "font-size": ["leading"],
      "fvn-normal": ["fvn-ordinal", "fvn-slashed-zero", "fvn-figure", "fvn-spacing", "fvn-fraction"],
      "fvn-ordinal": ["fvn-normal"],
      "fvn-slashed-zero": ["fvn-normal"],
      "fvn-figure": ["fvn-normal"],
      "fvn-spacing": ["fvn-normal"],
      "fvn-fraction": ["fvn-normal"],
      "line-clamp": ["display", "overflow"],
      rounded: ["rounded-s", "rounded-e", "rounded-t", "rounded-r", "rounded-b", "rounded-l", "rounded-ss", "rounded-se", "rounded-ee", "rounded-es", "rounded-tl", "rounded-tr", "rounded-br", "rounded-bl"],
      "rounded-s": ["rounded-ss", "rounded-es"],
      "rounded-e": ["rounded-se", "rounded-ee"],
      "rounded-t": ["rounded-tl", "rounded-tr"],
      "rounded-r": ["rounded-tr", "rounded-br"],
      "rounded-b": ["rounded-br", "rounded-bl"],
      "rounded-l": ["rounded-tl", "rounded-bl"],
      "border-spacing": ["border-spacing-x", "border-spacing-y"],
      "border-w": ["border-w-x", "border-w-y", "border-w-s", "border-w-e", "border-w-bs", "border-w-be", "border-w-t", "border-w-r", "border-w-b", "border-w-l"],
      "border-w-x": ["border-w-r", "border-w-l"],
      "border-w-y": ["border-w-t", "border-w-b"],
      "border-color": ["border-color-x", "border-color-y", "border-color-s", "border-color-e", "border-color-bs", "border-color-be", "border-color-t", "border-color-r", "border-color-b", "border-color-l"],
      "border-color-x": ["border-color-r", "border-color-l"],
      "border-color-y": ["border-color-t", "border-color-b"],
      translate: ["translate-x", "translate-y", "translate-none"],
      "translate-none": ["translate", "translate-x", "translate-y", "translate-z"],
      "scroll-m": ["scroll-mx", "scroll-my", "scroll-ms", "scroll-me", "scroll-mbs", "scroll-mbe", "scroll-mt", "scroll-mr", "scroll-mb", "scroll-ml"],
      "scroll-mx": ["scroll-mr", "scroll-ml"],
      "scroll-my": ["scroll-mt", "scroll-mb"],
      "scroll-p": ["scroll-px", "scroll-py", "scroll-ps", "scroll-pe", "scroll-pbs", "scroll-pbe", "scroll-pt", "scroll-pr", "scroll-pb", "scroll-pl"],
      "scroll-px": ["scroll-pr", "scroll-pl"],
      "scroll-py": ["scroll-pt", "scroll-pb"],
      touch: ["touch-x", "touch-y", "touch-pz"],
      "touch-x": ["touch"],
      "touch-y": ["touch"],
      "touch-pz": ["touch"]
    },
    conflictingClassGroupModifiers: {
      "font-size": ["leading"]
    },
    postfixLookupClassGroups: ["container-type"],
    orderSensitiveModifiers: ["*", "**", "after", "backdrop", "before", "details-content", "file", "first-letter", "first-line", "marker", "placeholder", "selection"]
  };
}, Ra = /* @__PURE__ */ oa(ka);
function ne(...e) {
  return Ra(Vr(e));
}
function Pa({ active: e, onChange: t, showAdmin: n }) {
  const r = fe(), o = Wr(), s = n ?? (o == null ? void 0 : o.role) === "admin", i = [
    { key: "today", label: r("nav.feed"), icon: Ai },
    { key: "children", label: r("nav.children"), icon: Di }
  ];
  return s && i.push({ key: "admin", label: r("nav.admin"), icon: Ii }), /* @__PURE__ */ d("nav", { className: "fixed inset-x-0 bottom-0 z-20 border-t border-border/70 bg-background/70 pb-[env(safe-area-inset-bottom)] backdrop-blur-xl", children: /* @__PURE__ */ d("div", { className: "mx-auto flex max-w-2xl justify-around", children: i.map((a) => {
    const c = e === a.key, l = a.icon;
    return /* @__PURE__ */ R(
      "button",
      {
        onClick: () => t(a.key),
        "aria-current": c ? "page" : void 0,
        className: ne(
          "flex min-h-[52px] flex-1 flex-col items-center justify-center gap-0.5 px-2 pt-1.5 text-[11px] font-medium transition-colors",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring",
          c ? "text-primary" : "text-muted-foreground hover:text-foreground"
        ),
        children: [
          /* @__PURE__ */ d(l, { className: "size-6", strokeWidth: c ? 2.4 : 2, "aria-hidden": !0 }),
          a.label
        ]
      },
      a.key
    );
  }) }) });
}
function Y(e, t, { checkForDefaultPrevented: n = !0 } = {}) {
  return function(o) {
    if (e == null || e(o), n === !1 || !o || !o.defaultPrevented)
      return t == null ? void 0 : t(o);
  };
}
function nt(e, t = []) {
  let n = [];
  function r(s, i) {
    const a = u.createContext(i);
    a.displayName = s + "Context";
    const c = n.length;
    n = [...n, i];
    const l = (m) => {
      var b;
      const { scope: h, children: g, ...w } = m, p = ((b = h == null ? void 0 : h[e]) == null ? void 0 : b[c]) || a, x = u.useMemo(() => w, Object.values(w));
      return /* @__PURE__ */ d(p.Provider, { value: x, children: g });
    };
    l.displayName = s + "Provider";
    function f(m, h, g = {}) {
      var b;
      const { optional: w = !1 } = g, p = ((b = h == null ? void 0 : h[e]) == null ? void 0 : b[c]) || a, x = u.useContext(p);
      if (x) return x;
      if (i !== void 0) return i;
      if (!w)
        throw new Error(`\`${m}\` must be used within \`${s}\``);
    }
    return [l, f];
  }
  const o = () => {
    const s = n.map((i) => u.createContext(i));
    return function(a) {
      const c = (a == null ? void 0 : a[e]) || s;
      return u.useMemo(
        () => ({ [`__scope${e}`]: { ...a, [e]: c } }),
        [a, c]
      );
    };
  };
  return o.scopeName = e, [r, Aa(o, ...t)];
}
function Aa(...e) {
  const t = e[0];
  if (e.length === 1) return t;
  const n = () => {
    const r = e.map((o) => ({
      useScope: o(),
      scopeName: o.scopeName
    }));
    return function(s) {
      const i = r.reduce((a, { useScope: c, scopeName: l }) => {
        const m = c(s)[`__scope${l}`];
        return { ...a, ...m };
      }, {});
      return u.useMemo(() => ({ [`__scope${t.scopeName}`]: i }), [i]);
    };
  };
  return n.scopeName = t.scopeName, n;
}
function sr(e, t) {
  if (typeof e == "function")
    return e(t);
  e != null && (e.current = t);
}
function Ta(...e) {
  return (t) => {
    let n = !1;
    const r = e.map((o) => {
      const s = sr(o, t);
      return !n && typeof s == "function" && (n = !0), s;
    });
    if (n)
      return () => {
        for (let o = 0; o < r.length; o++) {
          const s = r[o];
          typeof s == "function" ? s() : sr(e[o], null);
        }
      };
  };
}
function re(...e) {
  return u.useCallback(Ta(...e), e);
}
// @__NO_SIDE_EFFECTS__
function ut(e) {
  const t = u.forwardRef((n, r) => {
    let { children: o, ...s } = n, i = null, a = !1;
    const c = [];
    ir(o) && typeof St == "function" && (o = St(o._payload)), u.Children.forEach(o, (h) => {
      var g;
      if (Da(h)) {
        a = !0;
        const w = h;
        let p = "child" in w.props ? w.props.child : w.props.children;
        ir(p) && typeof St == "function" && (p = St(p._payload)), i = Oa(w, p), c.push((g = i == null ? void 0 : i.props) == null ? void 0 : g.children);
      } else
        c.push(h);
    }), i ? i = u.cloneElement(i, void 0, c) : (
      // A `Slottable` was found but it didn't resolve to a single element (e.g.
      // it wrapped multiple elements, text, or a render-prop `child` that
      // wasn't an element). Don't fall back to treating the `Slottable` wrapper
      // itself as the slot target — throw a descriptive error below instead.
      !a && u.Children.count(o) === 1 && u.isValidElement(o) && (i = o)
    );
    const l = i ? La(i) : void 0, f = re(r, l);
    if (!i) {
      if (o || o === 0)
        throw new Error(
          a ? Wa(e) : $a(e)
        );
      return o;
    }
    const m = Ma(s, i.props ?? {});
    return i.type !== u.Fragment && (m.ref = r ? f : l), u.cloneElement(i, m);
  });
  return t.displayName = `${e}.Slot`, t;
}
var _a = /* @__PURE__ */ ut("Slot"), Ia = Symbol.for("radix.slottable"), Oa = (e, t) => {
  if ("child" in e.props) {
    const n = e.props.child;
    return u.isValidElement(n) ? u.cloneElement(n, void 0, e.props.children(n.props.children)) : null;
  }
  return u.isValidElement(t) ? t : null;
};
function Ma(e, t) {
  const n = { ...t };
  for (const r in t) {
    const o = e[r], s = t[r];
    /^on[A-Z]/.test(r) ? o && s ? n[r] = (...a) => {
      const c = s(...a);
      return o(...a), c;
    } : o && (n[r] = o) : r === "style" ? n[r] = { ...o, ...s } : r === "className" && (n[r] = [o, s].filter(Boolean).join(" "));
  }
  return { ...e, ...n };
}
function La(e) {
  var r, o;
  let t = (r = Object.getOwnPropertyDescriptor(e.props, "ref")) == null ? void 0 : r.get, n = t && "isReactWarning" in t && t.isReactWarning;
  return n ? e.ref : (t = (o = Object.getOwnPropertyDescriptor(e, "ref")) == null ? void 0 : o.get, n = t && "isReactWarning" in t && t.isReactWarning, n ? e.props.ref : e.props.ref || e.ref);
}
function Da(e) {
  return u.isValidElement(e) && typeof e.type == "function" && "__radixId" in e.type && e.type.__radixId === Ia;
}
var Fa = Symbol.for("react.lazy");
function ir(e) {
  return e != null && typeof e == "object" && "$$typeof" in e && e.$$typeof === Fa && "_payload" in e && za(e._payload);
}
function za(e) {
  return typeof e == "object" && e !== null && "then" in e;
}
var $a = (e) => `${e} failed to slot onto its children. Expected a single React element child or \`Slottable\`.`, Wa = (e) => `${e} failed to slot onto its \`Slottable\`. Expected \`Slottable\` to receive a single React element child.`, St = u[" use ".trim().toString()];
function ro(e) {
  const t = e + "CollectionProvider", [n, r] = nt(t), [o, s] = n(
    t,
    { collectionRef: { current: null }, itemMap: /* @__PURE__ */ new Map() }
  ), i = (p) => {
    const { scope: x, children: b } = p, y = u.useRef(null), v = u.useRef(/* @__PURE__ */ new Map()).current;
    return /* @__PURE__ */ d(o, { scope: x, itemMap: v, collectionRef: y, children: b });
  };
  i.displayName = t;
  const a = e + "CollectionSlot", c = /* @__PURE__ */ ut(a), l = u.forwardRef(
    (p, x) => {
      const { scope: b, children: y } = p, v = s(a, b), S = re(x, v.collectionRef);
      return /* @__PURE__ */ d(c, { ref: S, children: y });
    }
  );
  l.displayName = a;
  const f = e + "CollectionItemSlot", m = "data-radix-collection-item", h = /* @__PURE__ */ ut(f), g = u.forwardRef(
    (p, x) => {
      const { scope: b, children: y, ...v } = p, S = u.useRef(null), E = re(x, S), C = s(f, b);
      return u.useEffect(() => (C.itemMap.set(S, { ref: S, ...v }), () => void C.itemMap.delete(S))), /* @__PURE__ */ d(h, { [m]: "", ref: E, children: y });
    }
  );
  g.displayName = f;
  function w(p) {
    const x = s(e + "CollectionConsumer", p);
    return u.useCallback(() => {
      const y = x.collectionRef.current;
      if (!y) return [];
      const v = Array.from(y.querySelectorAll(`[${m}]`));
      return Array.from(x.itemMap.values()).sort(
        (C, k) => v.indexOf(C.ref.current) - v.indexOf(k.ref.current)
      );
    }, [x.collectionRef, x.itemMap]);
  }
  return [
    { Provider: i, Slot: l, ItemSlot: g },
    w,
    r
  ];
}
var se = globalThis != null && globalThis.document ? u.useLayoutEffect : () => {
}, Ba = u[" useId ".trim().toString()] || (() => {
}), Va = 0;
function bt(e) {
  const [t, n] = u.useState(Ba());
  return se(() => {
    n((r) => r ?? String(Va++));
  }, [e]), t ? `radix-${t}` : "";
}
var Ha = [
  "a",
  "button",
  "div",
  "form",
  "h2",
  "h3",
  "img",
  "input",
  "label",
  "li",
  "nav",
  "ol",
  "p",
  "select",
  "span",
  "svg",
  "ul"
], q = Ha.reduce((e, t) => {
  const n = /* @__PURE__ */ ut(`Primitive.${t}`), r = u.forwardRef((o, s) => {
    const { asChild: i, ...a } = o, c = i ? n : t;
    return typeof window < "u" && (window[Symbol.for("radix-ui")] = !0), /* @__PURE__ */ d(c, { ...a, ref: s });
  });
  return r.displayName = `Primitive.${t}`, { ...e, [t]: r };
}, {});
function Ua(e, t) {
  e && vt.flushSync(() => e.dispatchEvent(t));
}
function xe(e) {
  const t = u.useRef(e);
  return u.useEffect(() => {
    t.current = e;
  }), u.useMemo(() => (...n) => {
    var r;
    return (r = t.current) == null ? void 0 : r.call(t, ...n);
  }, []);
}
var ja = u[" useInsertionEffect ".trim().toString()] || se;
function ft({
  prop: e,
  defaultProp: t,
  onChange: n = () => {
  },
  caller: r
}) {
  const [o, s, i] = Ga({
    defaultProp: t,
    onChange: n
  }), a = e !== void 0, c = a ? e : o;
  {
    const f = u.useRef(e !== void 0);
    u.useEffect(() => {
      const m = f.current;
      m !== a && console.warn(
        `${r} is changing from ${m ? "controlled" : "uncontrolled"} to ${a ? "controlled" : "uncontrolled"}. Components should not switch from controlled to uncontrolled (or vice versa). Decide between using a controlled or uncontrolled value for the lifetime of the component.`
      ), f.current = a;
    }, [a, r]);
  }
  const l = u.useCallback(
    (f) => {
      var m;
      if (a) {
        const h = Ka(f) ? f(e) : f;
        h !== e && ((m = i.current) == null || m.call(i, h));
      } else
        s(f);
    },
    [a, e, s, i]
  );
  return [c, l];
}
function Ga({
  defaultProp: e,
  onChange: t
}) {
  const [n, r] = u.useState(e), o = u.useRef(n), s = u.useRef(t);
  return ja(() => {
    s.current = t;
  }, [t]), u.useEffect(() => {
    var i;
    o.current !== n && ((i = s.current) == null || i.call(s, n), o.current = n);
  }, [n, o]), [n, r, s];
}
function Ka(e) {
  return typeof e == "function";
}
var qa = u.createContext(void 0);
function _n(e) {
  const t = u.useContext(qa);
  return e || t || "ltr";
}
var rn = !1;
function Ya() {
  const [e, t] = u.useState(rn);
  return u.useEffect(() => {
    rn || (rn = !0, t(!0));
  }, []), e;
}
var oo = u[" useSyncExternalStore ".trim().toString()];
function Xa() {
  return () => {
  };
}
function Za() {
  return oo(
    Xa,
    () => !0,
    () => !1
  );
}
var Qa = typeof oo == "function" ? Za : Ya, on = "rovingFocusGroup.onEntryFocus", Ja = { bubbles: !1, cancelable: !0 }, yt = "RovingFocusGroup", [vn, so, ec] = ro(yt), [tc, io] = nt(
  yt,
  [ec]
), [nc, rc] = tc(yt), ao = u.forwardRef(
  (e, t) => /* @__PURE__ */ d(vn.Provider, { scope: e.__scopeRovingFocusGroup, children: /* @__PURE__ */ d(vn.Slot, { scope: e.__scopeRovingFocusGroup, children: /* @__PURE__ */ d(oc, { ...e, ref: t }) }) })
);
ao.displayName = yt;
var oc = u.forwardRef((e, t) => {
  const {
    __scopeRovingFocusGroup: n,
    orientation: r,
    loop: o = !1,
    dir: s,
    currentTabStopId: i,
    defaultCurrentTabStopId: a,
    onCurrentTabStopIdChange: c,
    onEntryFocus: l,
    preventScrollOnEntryFocus: f = !1,
    ...m
  } = e, h = u.useRef(null), g = re(t, h), w = _n(s), [p, x] = ft({
    prop: i,
    defaultProp: a ?? null,
    onChange: c,
    caller: yt
  }), [b, y] = u.useState(!1), v = xe(l), S = so(n), E = u.useRef(!1), [C, k] = u.useState(0);
  return u.useEffect(() => {
    const N = h.current;
    if (N)
      return N.addEventListener(on, v), () => N.removeEventListener(on, v);
  }, [v]), /* @__PURE__ */ d(
    nc,
    {
      scope: n,
      orientation: r,
      dir: w,
      loop: o,
      currentTabStopId: p,
      onItemFocus: u.useCallback(
        (N) => x(N),
        [x]
      ),
      onItemShiftTab: u.useCallback(() => y(!0), []),
      onFocusableItemAdd: u.useCallback(
        () => k((N) => N + 1),
        []
      ),
      onFocusableItemRemove: u.useCallback(
        () => k((N) => N - 1),
        []
      ),
      children: /* @__PURE__ */ d(
        q.div,
        {
          tabIndex: b || C === 0 ? -1 : 0,
          "data-orientation": r,
          ...m,
          ref: g,
          style: { outline: "none", ...e.style },
          onMouseDown: Y(e.onMouseDown, () => {
            E.current = !0;
          }),
          onFocus: Y(e.onFocus, (N) => {
            const L = !E.current;
            if (N.target === N.currentTarget && L && !b) {
              const F = new CustomEvent(on, Ja);
              if (N.currentTarget.dispatchEvent(F), !F.defaultPrevented) {
                const _ = S().filter((O) => O.focusable), B = _.find((O) => O.active), $ = _.find((O) => O.id === p), U = [B, $, ..._].filter(
                  Boolean
                ).map((O) => O.ref.current);
                uo(U, f);
              }
            }
            E.current = !1;
          }),
          onBlur: Y(e.onBlur, () => y(!1))
        }
      )
    }
  );
}), co = "RovingFocusGroupItem", lo = u.forwardRef(
  (e, t) => {
    const {
      __scopeRovingFocusGroup: n,
      focusable: r = !0,
      active: o = !1,
      tabStopId: s,
      children: i,
      ...a
    } = e, c = bt(), l = s || c, f = rc(co, n), m = f.currentTabStopId === l, h = so(n), { onFocusableItemAdd: g, onFocusableItemRemove: w, currentTabStopId: p } = f, x = Qa();
    return se(() => {
      if (!(!x || !r))
        return g(), () => w();
    }, [x, r, g, w]), u.useEffect(() => {
      if (!(x || !r))
        return g(), () => w();
    }, [x, r, g, w]), /* @__PURE__ */ d(
      vn.ItemSlot,
      {
        scope: n,
        id: l,
        focusable: r,
        active: o,
        children: /* @__PURE__ */ d(
          q.span,
          {
            tabIndex: m ? 0 : -1,
            "data-orientation": f.orientation,
            ...a,
            ref: t,
            onMouseDown: Y(e.onMouseDown, (b) => {
              r ? f.onItemFocus(l) : b.preventDefault();
            }),
            onFocus: Y(e.onFocus, () => f.onItemFocus(l)),
            onKeyDown: Y(e.onKeyDown, (b) => {
              if (b.key === "Tab" && b.shiftKey) {
                f.onItemShiftTab();
                return;
              }
              if (b.target !== b.currentTarget) return;
              const y = ac(b, f.orientation, f.dir);
              if (y !== void 0) {
                if (b.metaKey || b.ctrlKey || b.altKey || b.shiftKey) return;
                b.preventDefault();
                let S = h().filter((E) => E.focusable).map((E) => E.ref.current);
                if (y === "last") S.reverse();
                else if (y === "prev" || y === "next") {
                  y === "prev" && S.reverse();
                  const E = S.indexOf(b.currentTarget);
                  S = f.loop ? cc(S, E + 1) : S.slice(E + 1);
                }
                setTimeout(() => uo(S));
              }
            }),
            children: typeof i == "function" ? i({ isCurrentTabStop: m, hasTabStop: p != null }) : i
          }
        )
      }
    );
  }
);
lo.displayName = co;
var sc = {
  ArrowLeft: "prev",
  ArrowUp: "prev",
  ArrowRight: "next",
  ArrowDown: "next",
  PageUp: "first",
  Home: "first",
  PageDown: "last",
  End: "last"
};
function ic(e, t) {
  return t !== "rtl" ? e : e === "ArrowLeft" ? "ArrowRight" : e === "ArrowRight" ? "ArrowLeft" : e;
}
function ac(e, t, n) {
  const r = ic(e.key, n);
  if (!(t === "vertical" && ["ArrowLeft", "ArrowRight"].includes(r)) && !(t === "horizontal" && ["ArrowUp", "ArrowDown"].includes(r)))
    return sc[r];
}
function uo(e, t = !1) {
  const n = document.activeElement;
  for (const r of e)
    if (r === n || (r.focus({ preventScroll: t }), document.activeElement !== n)) return;
}
function cc(e, t) {
  return e.map((n, r) => e[(t + r) % e.length]);
}
var lc = ao, dc = lo;
function uc(e, t) {
  return u.useReducer((n, r) => t[n][r] ?? n, e);
}
var In = (e) => {
  const { present: t, children: n } = e, r = fc(t), o = typeof n == "function" ? n({ present: r.isPresent }) : u.Children.only(n), s = mc(r.ref, pc(o));
  return typeof n == "function" || r.isPresent ? u.cloneElement(o, { ref: s }) : null;
};
In.displayName = "Presence";
function fc(e) {
  const [t, n] = u.useState(), r = u.useRef(null), o = u.useRef(e), s = u.useRef("none"), i = u.useRef(void 0), a = e ? "mounted" : "unmounted", [c, l] = uc(a, {
    mounted: {
      UNMOUNT: "unmounted",
      ANIMATION_OUT: "unmountSuspended"
    },
    unmountSuspended: {
      MOUNT: "mounted",
      ANIMATION_END: "unmounted"
    },
    unmounted: {
      MOUNT: "mounted"
    }
  });
  return u.useEffect(() => {
    c === "mounted" ? (s.current = i.current ?? ct(r.current), i.current = void 0) : s.current = "none";
  }, [c]), se(() => {
    const f = r.current, m = o.current;
    if (m !== e) {
      const g = s.current, w = ct(f);
      e ? (i.current = w, l("MOUNT")) : w === "none" || (f == null ? void 0 : f.display) === "none" ? l("UNMOUNT") : l(m && g !== w ? "ANIMATION_OUT" : "UNMOUNT"), o.current = e;
    }
  }, [e, l]), se(() => {
    if (t) {
      let f;
      const m = t.ownerDocument.defaultView ?? window, h = (w) => {
        const x = ct(r.current).includes(CSS.escape(w.animationName));
        if (w.target === t && x && (l("ANIMATION_END"), !o.current)) {
          const b = t.style.animationFillMode;
          t.style.animationFillMode = "forwards", f = m.setTimeout(() => {
            t.style.animationFillMode === "forwards" && (t.style.animationFillMode = b);
          });
        }
      }, g = (w) => {
        w.target === t && (s.current = ct(r.current));
      };
      return t.addEventListener("animationstart", g), t.addEventListener("animationcancel", h), t.addEventListener("animationend", h), () => {
        m.clearTimeout(f), t.removeEventListener("animationstart", g), t.removeEventListener("animationcancel", h), t.removeEventListener("animationend", h);
      };
    } else
      l("ANIMATION_END");
  }, [t, l]), {
    isPresent: ["mounted", "unmountSuspended"].includes(c),
    ref: u.useCallback((f) => {
      if (f) {
        const m = getComputedStyle(f);
        r.current = m, i.current = ct(m);
      } else
        r.current = null;
      n(f);
    }, [])
  };
}
function ar(e, t) {
  if (typeof e == "function")
    return e(t);
  e != null && (e.current = t);
}
function mc(...e) {
  const t = u.useRef(e);
  return t.current = e, u.useCallback((n) => {
    const r = t.current;
    let o = !1;
    const s = r.map((i) => {
      const a = ar(i, n);
      return !o && typeof a == "function" && (o = !0), a;
    });
    if (o)
      return () => {
        for (let i = 0; i < s.length; i++) {
          const a = s[i];
          typeof a == "function" ? a() : ar(r[i], null);
        }
      };
  }, []);
}
function ct(e) {
  return (e == null ? void 0 : e.animationName) || "none";
}
function pc(e) {
  var r, o;
  let t = (r = Object.getOwnPropertyDescriptor(e.props, "ref")) == null ? void 0 : r.get, n = t && "isReactWarning" in t && t.isReactWarning;
  return n ? e.ref : (t = (o = Object.getOwnPropertyDescriptor(e, "ref")) == null ? void 0 : o.get, n = t && "isReactWarning" in t && t.isReactWarning, n ? e.props.ref : e.props.ref || e.ref);
}
var Vt = "Tabs", [hc] = nt(Vt, [
  io
]), fo = io(), [gc, On] = hc(Vt), mo = u.forwardRef(
  (e, t) => {
    const {
      __scopeTabs: n,
      value: r,
      onValueChange: o,
      defaultValue: s,
      orientation: i = "horizontal",
      dir: a,
      activationMode: c = "automatic",
      ...l
    } = e, f = _n(a), [m, h] = ft({
      prop: r,
      onChange: o,
      defaultProp: s ?? "",
      caller: Vt
    });
    return /* @__PURE__ */ d(
      gc,
      {
        scope: n,
        baseId: bt(),
        value: m,
        onValueChange: h,
        orientation: i,
        dir: f,
        activationMode: c,
        children: /* @__PURE__ */ d(
          q.div,
          {
            dir: f,
            "data-orientation": i,
            ...l,
            ref: t
          }
        )
      }
    );
  }
);
mo.displayName = Vt;
var po = "TabsList", ho = u.forwardRef(
  (e, t) => {
    const { __scopeTabs: n, loop: r = !0, ...o } = e, s = On(po, n), i = fo(n);
    return /* @__PURE__ */ d(
      lc,
      {
        asChild: !0,
        ...i,
        orientation: s.orientation,
        dir: s.dir,
        loop: r,
        children: /* @__PURE__ */ d(
          q.div,
          {
            role: "tablist",
            "aria-orientation": s.orientation,
            ...o,
            ref: t
          }
        )
      }
    );
  }
);
ho.displayName = po;
var go = "TabsTrigger", vo = u.forwardRef(
  (e, t) => {
    const { __scopeTabs: n, value: r, disabled: o = !1, ...s } = e, i = On(go, n), a = fo(n), c = wo(i.baseId, r), l = xo(i.baseId, r), f = r === i.value;
    return /* @__PURE__ */ d(
      dc,
      {
        asChild: !0,
        ...a,
        focusable: !o,
        active: f,
        children: /* @__PURE__ */ d(
          q.button,
          {
            type: "button",
            role: "tab",
            "aria-selected": f,
            "aria-controls": l,
            "data-state": f ? "active" : "inactive",
            "data-disabled": o ? "" : void 0,
            disabled: o,
            id: c,
            ...s,
            ref: t,
            onMouseDown: Y(e.onMouseDown, (m) => {
              !o && m.button === 0 && m.ctrlKey === !1 ? i.onValueChange(r) : m.preventDefault();
            }),
            onKeyDown: Y(e.onKeyDown, (m) => {
              o || m.target !== m.currentTarget || [" ", "Enter"].includes(m.key) && i.onValueChange(r);
            }),
            onFocus: Y(e.onFocus, () => {
              const m = i.activationMode !== "manual";
              !f && !o && m && i.onValueChange(r);
            })
          }
        )
      }
    );
  }
);
vo.displayName = go;
var bo = "TabsContent", yo = u.forwardRef(
  (e, t) => {
    const { __scopeTabs: n, value: r, forceMount: o, children: s, ...i } = e, a = On(bo, n), c = wo(a.baseId, r), l = xo(a.baseId, r), f = r === a.value, m = u.useRef(f);
    return u.useEffect(() => {
      const h = requestAnimationFrame(() => m.current = !1);
      return () => cancelAnimationFrame(h);
    }, []), /* @__PURE__ */ d(In, { present: o || f, children: ({ present: h }) => /* @__PURE__ */ d(
      q.div,
      {
        "data-state": f ? "active" : "inactive",
        "data-orientation": a.orientation,
        role: "tabpanel",
        "aria-labelledby": c,
        hidden: !h,
        id: l,
        tabIndex: 0,
        ...i,
        ref: t,
        style: {
          ...e.style,
          animationDuration: m.current ? "0s" : void 0
        },
        children: h && s
      }
    ) });
  }
);
yo.displayName = bo;
function wo(e, t) {
  return `${e}-trigger-${t}`;
}
function xo(e, t) {
  return `${e}-content-${t}`;
}
var vc = mo, Co = ho, So = vo, No = yo;
const bc = vc, Eo = u.forwardRef(({ className: e, ...t }, n) => /* @__PURE__ */ d(
  Co,
  {
    ref: n,
    className: ne(
      "flex items-center gap-1 overflow-x-auto rounded-full bg-muted p-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
      e
    ),
    ...t
  }
));
Eo.displayName = Co.displayName;
const ko = u.forwardRef(({ className: e, ...t }, n) => /* @__PURE__ */ d(
  So,
  {
    ref: n,
    className: ne(
      "inline-flex shrink-0 items-center justify-center whitespace-nowrap rounded-full px-3.5 py-1.5 text-sm font-medium text-muted-foreground transition-all",
      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
      "data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm",
      e
    ),
    ...t
  }
));
ko.displayName = So.displayName;
const lt = u.forwardRef(({ className: e, ...t }, n) => /* @__PURE__ */ d(
  No,
  {
    ref: n,
    className: ne("focus-visible:outline-none", e),
    ...t
  }
));
lt.displayName = No.displayName;
function Ae() {
  const e = li();
  return {
    list: (t, n) => e(`care.${t}.list`, n ?? {}),
    get: (t, n) => e(`care.${t}.get`, { id: n }),
    /**
     * Call a `care.*` verb that doesn't follow the `<noun>.<action>`
     * shape (`create` / `update` / `archive` / `link` / `unlink` etc).
     * Pass the FULL verb including the action (e.g. `"center.create"`).
     */
    run: (t, n) => e(`care.${t}`, n)
  };
}
var yc = "Label", Ro = u.forwardRef((e, t) => /* @__PURE__ */ d(
  q.label,
  {
    ...e,
    ref: t,
    onMouseDown: (n) => {
      var o;
      n.target.closest("button, input, select, textarea") || ((o = e.onMouseDown) == null || o.call(e, n), !n.defaultPrevented && n.detail > 1 && n.preventDefault());
    }
  }
));
Ro.displayName = yc;
var Po = Ro;
const Ao = u.forwardRef(({ className: e, ...t }, n) => /* @__PURE__ */ d(
  Po,
  {
    ref: n,
    className: ne("text-sm font-medium leading-none text-foreground", e),
    ...t
  }
));
Ao.displayName = Po.displayName;
function te({
  label: e,
  required: t,
  hint: n,
  htmlFor: r,
  children: o
}) {
  return /* @__PURE__ */ R("div", { className: "space-y-1.5", children: [
    /* @__PURE__ */ R(Ao, { htmlFor: r, className: "flex items-baseline gap-2", children: [
      /* @__PURE__ */ R("span", { children: [
        e,
        t && /* @__PURE__ */ d("span", { className: "ml-0.5 text-destructive", children: "*" })
      ] }),
      n && /* @__PURE__ */ d("span", { className: "text-xs font-normal text-muted-foreground", children: n })
    ] }),
    o
  ] });
}
const cr = (e) => typeof e == "boolean" ? `${e}` : e === 0 ? "0" : e, lr = Vr, wc = (e, t) => (n) => {
  var r;
  if ((t == null ? void 0 : t.variants) == null) return lr(e, n == null ? void 0 : n.class, n == null ? void 0 : n.className);
  const { variants: o, defaultVariants: s } = t, i = Object.keys(o).map((l) => {
    const f = n == null ? void 0 : n[l], m = s == null ? void 0 : s[l];
    if (f === null) return null;
    const h = cr(f) || cr(m);
    return o[l][h];
  }), a = n && Object.entries(n).reduce((l, f) => {
    let [m, h] = f;
    return h === void 0 || (l[m] = h), l;
  }, {}), c = t == null || (r = t.compoundVariants) === null || r === void 0 ? void 0 : r.reduce((l, f) => {
    let { class: m, className: h, ...g } = f;
    return Object.entries(g).every((w) => {
      let [p, x] = w;
      return Array.isArray(x) ? x.includes({
        ...s,
        ...a
      }[p]) : {
        ...s,
        ...a
      }[p] === x;
    }) ? [
      ...l,
      m,
      h
    ] : l;
  }, []);
  return lr(e, i, c, n == null ? void 0 : n.class, n == null ? void 0 : n.className);
}, xc = wc(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-medium transition-all duration-200 ease-out active:scale-[0.98] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground shadow-sm hover:bg-primary/90",
        destructive: "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90",
        outline: "border border-border bg-card hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline"
      },
      size: {
        default: "h-11 px-5 py-2",
        sm: "h-9 rounded-lg px-3",
        lg: "h-12 rounded-2xl px-6 text-base",
        pill: "h-8 rounded-full px-3.5 text-xs",
        icon: "h-9 w-9 rounded-full"
      }
    },
    defaultVariants: { variant: "default", size: "default" }
  }
), ie = u.forwardRef(
  ({ className: e, variant: t, size: n, asChild: r = !1, ...o }, s) => /* @__PURE__ */ d(r ? _a : "button", { className: ne(xc({ variant: t, size: n, className: e })), ref: s, ...o })
);
ie.displayName = "Button";
const pe = u.forwardRef(
  ({ className: e, type: t, ...n }, r) => /* @__PURE__ */ d(
    "input",
    {
      type: t,
      ref: r,
      className: ne(
        "flex h-11 w-full rounded-xl border border-input bg-card px-4 py-2 text-base text-foreground shadow-sm transition-colors",
        "placeholder:text-muted-foreground",
        "focus-visible:outline-none focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40",
        "disabled:cursor-not-allowed disabled:opacity-50",
        e
      ),
      ...n
    }
  )
);
pe.displayName = "Input";
function Mn({
  segments: e,
  value: t,
  onChange: n,
  className: r,
  columns: o
}) {
  return /* @__PURE__ */ d(
    "div",
    {
      role: "group",
      className: ne("gap-0.5 rounded-xl bg-muted p-0.5", o ? "grid" : "inline-flex", r),
      style: o ? { gridTemplateColumns: `repeat(${o}, minmax(0, 1fr))` } : void 0,
      children: e.map((s) => {
        const i = s.value === t;
        return /* @__PURE__ */ d(
          "button",
          {
            type: "button",
            "aria-pressed": i,
            onClick: () => n(s.value),
            className: ne(
              "rounded-[0.625rem] px-3 py-1.5 text-sm font-medium transition-all duration-200 ease-out",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              i ? "bg-card text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
            ),
            children: s.label
          },
          s.value
        );
      })
    }
  );
}
function dr(e, [t, n]) {
  return Math.min(n, Math.max(t, e));
}
var Cc = "DismissableLayer", bn = "dismissableLayer.update", Sc = "dismissableLayer.pointerDownOutside", Nc = "dismissableLayer.focusOutside", ur, To = u.createContext({
  layers: /* @__PURE__ */ new Set(),
  layersWithOutsidePointerEventsDisabled: /* @__PURE__ */ new Set(),
  branches: /* @__PURE__ */ new Set(),
  // Outside elements that belong to a layer's own dismiss affordance (eg, a
  // dialog overlay). Pressing them should dismiss the layer regardless of
  // whether or not they stop propagation.
  //
  // See https://github.com/radix-ui/primitives/issues/3346
  dismissableSurfaces: /* @__PURE__ */ new Set()
}), _o = u.forwardRef(
  (e, t) => {
    const {
      disableOutsidePointerEvents: n = !1,
      deferPointerDownOutside: r = !1,
      onEscapeKeyDown: o,
      onPointerDownOutside: s,
      onFocusOutside: i,
      onInteractOutside: a,
      onDismiss: c,
      ...l
    } = e, f = u.useContext(To), [m, h] = u.useState(null), g = (m == null ? void 0 : m.ownerDocument) ?? (globalThis == null ? void 0 : globalThis.document), [, w] = u.useState({}), p = re(t, h), x = Array.from(f.layers), [b] = [
      ...f.layersWithOutsidePointerEventsDisabled
    ].slice(-1), y = b ? x.indexOf(b) : -1, v = m ? x.indexOf(m) : -1, S = f.layersWithOutsidePointerEventsDisabled.size > 0, E = v >= y, C = u.useRef(!1), k = Pc(
      (_) => {
        s == null || s(_), a == null || a(_), _.defaultPrevented || c == null || c();
      },
      {
        ownerDocument: g,
        deferPointerDownOutside: r,
        isDeferredPointerDownOutsideRef: C,
        dismissableSurfaces: f.dismissableSurfaces,
        shouldHandlePointerDownOutside: u.useCallback(
          (_) => {
            if (!(_ instanceof Node))
              return !1;
            const B = [...f.branches].some(
              ($) => $.contains(_)
            );
            return E && !B;
          },
          [f.branches, E]
        )
      }
    ), N = Ac((_) => {
      if (r && C.current)
        return;
      const B = _.target;
      [...f.branches].some((H) => H.contains(B)) || (i == null || i(_), a == null || a(_), _.defaultPrevented || c == null || c());
    }, g), L = m ? v === x.length - 1 : !1, F = xe((_) => {
      _.key === "Escape" && (o == null || o(_), !_.defaultPrevented && c && (_.preventDefault(), c()));
    });
    return u.useEffect(() => {
      if (L)
        return g.addEventListener("keydown", F, { capture: !0 }), () => g.removeEventListener("keydown", F, { capture: !0 });
    }, [g, L, F]), u.useEffect(() => {
      if (m)
        return n && (f.layersWithOutsidePointerEventsDisabled.size === 0 && (ur = g.body.style.pointerEvents, g.body.style.pointerEvents = "none"), f.layersWithOutsidePointerEventsDisabled.add(m)), f.layers.add(m), fr(), () => {
          n && (f.layersWithOutsidePointerEventsDisabled.delete(m), f.layersWithOutsidePointerEventsDisabled.size === 0 && (g.body.style.pointerEvents = ur));
        };
    }, [m, g, n, f]), u.useEffect(() => () => {
      m && (f.layers.delete(m), f.layersWithOutsidePointerEventsDisabled.delete(m), fr());
    }, [m, f]), u.useEffect(() => {
      const _ = () => w({});
      return document.addEventListener(bn, _), () => document.removeEventListener(bn, _);
    }, []), /* @__PURE__ */ d(
      q.div,
      {
        ...l,
        ref: p,
        style: {
          pointerEvents: S ? E ? "auto" : "none" : void 0,
          ...e.style
        },
        onFocusCapture: Y(e.onFocusCapture, N.onFocusCapture),
        onBlurCapture: Y(e.onBlurCapture, N.onBlurCapture),
        onPointerDownCapture: Y(
          e.onPointerDownCapture,
          k.onPointerDownCapture
        )
      }
    );
  }
);
_o.displayName = Cc;
var Ec = "DismissableLayerBranch", kc = u.forwardRef((e, t) => {
  const n = u.useContext(To), r = u.useRef(null), o = re(t, r);
  return u.useEffect(() => {
    const s = r.current;
    if (s)
      return n.branches.add(s), () => {
        n.branches.delete(s);
      };
  }, [n.branches]), /* @__PURE__ */ d(q.div, { ...e, ref: o });
});
kc.displayName = Ec;
var Rc = () => !0;
function Pc(e, t) {
  const {
    ownerDocument: n = globalThis == null ? void 0 : globalThis.document,
    deferPointerDownOutside: r = !1,
    isDeferredPointerDownOutsideRef: o,
    dismissableSurfaces: s,
    shouldHandlePointerDownOutside: i = Rc
  } = t, a = xe(e), c = u.useRef(!1), l = u.useRef(!1), f = u.useRef(/* @__PURE__ */ new Map()), m = u.useRef(() => {
  });
  return u.useEffect(() => {
    function h() {
      l.current = !1, o.current = !1, f.current.clear();
    }
    function g() {
      return Array.from(f.current.values()).some(Boolean);
    }
    function w(v) {
      if (!l.current)
        return;
      const S = v.target;
      S instanceof Node && [...s].some((C) => C.contains(S)) || f.current.set(v.type, !0), v.type === "click" && window.setTimeout(() => {
        l.current && m.current();
      }, 0);
    }
    function p(v) {
      l.current && f.current.set(v.type, !1);
    }
    const x = (v) => {
      if (v.target && !c.current) {
        let S = function() {
          n.removeEventListener("click", m.current);
          const C = g();
          h(), C || Io(
            Sc,
            a,
            E,
            { discrete: !0 }
          );
        };
        if (!i(v.target)) {
          n.removeEventListener("click", m.current), h(), c.current = !1;
          return;
        }
        const E = { originalEvent: v };
        l.current = !0, o.current = r && v.button === 0, f.current.clear(), !r || v.button !== 0 ? S() : (n.removeEventListener("click", m.current), m.current = S, n.addEventListener("click", m.current, { once: !0 }));
      } else
        n.removeEventListener("click", m.current), h();
      c.current = !1;
    }, b = [
      "pointerup",
      "mousedown",
      "mouseup",
      "touchstart",
      "touchend",
      "click"
    ];
    for (const v of b)
      n.addEventListener(v, w, !0), n.addEventListener(v, p);
    const y = window.setTimeout(() => {
      n.addEventListener("pointerdown", x);
    }, 0);
    return () => {
      window.clearTimeout(y), n.removeEventListener("pointerdown", x), n.removeEventListener("click", m.current);
      for (const v of b)
        n.removeEventListener(v, w, !0), n.removeEventListener(v, p);
    };
  }, [
    n,
    a,
    r,
    o,
    s,
    i
  ]), {
    // ensures we check React component tree (not just DOM tree)
    onPointerDownCapture: () => c.current = !0
  };
}
function Ac(e, t = globalThis == null ? void 0 : globalThis.document) {
  const n = xe(e), r = u.useRef(!1);
  return u.useEffect(() => {
    const o = (s) => {
      s.target && !r.current && Io(Nc, n, { originalEvent: s }, {
        discrete: !1
      });
    };
    return t.addEventListener("focusin", o), () => t.removeEventListener("focusin", o);
  }, [t, n]), {
    onFocusCapture: () => r.current = !0,
    onBlurCapture: () => r.current = !1
  };
}
function fr() {
  const e = new CustomEvent(bn);
  document.dispatchEvent(e);
}
function Io(e, t, n, { discrete: r }) {
  const o = n.originalEvent.target, s = new CustomEvent(e, { bubbles: !1, cancelable: !0, detail: n });
  t && o.addEventListener(e, t, { once: !0 }), r ? Ua(o, s) : o.dispatchEvent(s);
}
var Nt = 0, be = null;
function Tc() {
  u.useEffect(() => {
    be || (be = { start: mr(), end: mr() });
    const { start: e, end: t } = be;
    return document.body.firstElementChild !== e && document.body.insertAdjacentElement("afterbegin", e), document.body.lastElementChild !== t && document.body.insertAdjacentElement("beforeend", t), Nt++, () => {
      Nt === 1 && (be == null || be.start.remove(), be == null || be.end.remove(), be = null), Nt = Math.max(0, Nt - 1);
    };
  }, []);
}
function mr() {
  const e = document.createElement("span");
  return e.setAttribute("data-radix-focus-guard", ""), e.tabIndex = 0, e.style.outline = "none", e.style.opacity = "0", e.style.position = "fixed", e.style.pointerEvents = "none", e;
}
var sn = "focusScope.autoFocusOnMount", an = "focusScope.autoFocusOnUnmount", pr = { bubbles: !1, cancelable: !0 }, _c = "FocusScope", Oo = u.forwardRef((e, t) => {
  const {
    loop: n = !1,
    trapped: r = !1,
    onMountAutoFocus: o,
    onUnmountAutoFocus: s,
    ...i
  } = e, [a, c] = u.useState(null), l = xe(o), f = xe(s), m = u.useRef(null), h = re(t, c), g = u.useRef({
    paused: !1,
    pause() {
      this.paused = !0;
    },
    resume() {
      this.paused = !1;
    }
  }).current;
  u.useEffect(() => {
    if (r) {
      let p = function(v) {
        if (g.paused || !a) return;
        const S = v.target;
        a.contains(S) ? m.current = S : Ie(m.current, { select: !0 });
      }, x = function(v) {
        if (g.paused || !a) return;
        const S = v.relatedTarget;
        S !== null && (a.contains(S) || Ie(m.current, { select: !0 }));
      }, b = function(v) {
        if (document.activeElement === document.body)
          for (const E of v)
            E.removedNodes.length > 0 && Ie(a);
      };
      document.addEventListener("focusin", p), document.addEventListener("focusout", x);
      const y = new MutationObserver(b);
      return a && y.observe(a, { childList: !0, subtree: !0 }), () => {
        document.removeEventListener("focusin", p), document.removeEventListener("focusout", x), y.disconnect();
      };
    }
  }, [r, a, g.paused]), u.useEffect(() => {
    if (a) {
      gr.add(g);
      const p = document.activeElement;
      if (!a.contains(p)) {
        const b = new CustomEvent(sn, pr);
        a.addEventListener(sn, l), a.dispatchEvent(b), b.defaultPrevented || (Ic(Fc(Mo(a)), { select: !0 }), document.activeElement === p && Ie(a));
      }
      return () => {
        a.removeEventListener(sn, l), setTimeout(() => {
          const b = new CustomEvent(an, pr);
          a.addEventListener(an, f), a.dispatchEvent(b), b.defaultPrevented || Ie(p ?? document.body, { select: !0 }), a.removeEventListener(an, f), gr.remove(g);
        }, 0);
      };
    }
  }, [a, l, f, g]);
  const w = u.useCallback(
    (p) => {
      if (!n && !r || g.paused) return;
      const x = p.key === "Tab" && !p.altKey && !p.ctrlKey && !p.metaKey, b = document.activeElement;
      if (x && b) {
        const y = p.currentTarget, [v, S] = Oc(y);
        v && S ? !p.shiftKey && b === S ? (p.preventDefault(), n && Ie(v, { select: !0 })) : p.shiftKey && b === v && (p.preventDefault(), n && Ie(S, { select: !0 })) : b === y && p.preventDefault();
      }
    },
    [n, r, g.paused]
  );
  return /* @__PURE__ */ d(q.div, { tabIndex: -1, ...i, ref: h, onKeyDown: w });
});
Oo.displayName = _c;
function Ic(e, { select: t = !1 } = {}) {
  const n = document.activeElement;
  for (const r of e)
    if (Ie(r, { select: t }), document.activeElement !== n) return;
}
function Oc(e) {
  const t = Mo(e), n = hr(t, e), r = hr(t.reverse(), e);
  return [n, r];
}
function Mo(e) {
  const t = [], n = document.createTreeWalker(e, NodeFilter.SHOW_ELEMENT, {
    acceptNode: (r) => {
      const o = r.tagName === "INPUT" && r.type === "hidden";
      return r.disabled || r.hidden || o ? NodeFilter.FILTER_SKIP : r.tabIndex >= 0 ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_SKIP;
    }
  });
  for (; n.nextNode(); ) t.push(n.currentNode);
  return t;
}
function hr(e, t) {
  const n = typeof t.checkVisibility == "function" && t.checkVisibility({ checkVisibilityCSS: !0 });
  for (const r of e)
    if (!(n ? !r.checkVisibility({ checkVisibilityCSS: !0 }) : Mc(r, { upTo: t })))
      return r;
}
function Mc(e, { upTo: t }) {
  if (getComputedStyle(e).visibility === "hidden") return !0;
  for (; e; ) {
    if (t !== void 0 && e === t) return !1;
    if (getComputedStyle(e).display === "none") return !0;
    e = e.parentElement;
  }
  return !1;
}
function Lc(e) {
  return e instanceof HTMLInputElement && "select" in e;
}
function Ie(e, { select: t = !1 } = {}) {
  if (e && e.focus) {
    const n = document.activeElement;
    e.focus({ preventScroll: !0 }), e !== n && Lc(e) && t && e.select();
  }
}
var gr = Dc();
function Dc() {
  let e = [];
  return {
    add(t) {
      const n = e[0];
      t !== n && (n == null || n.pause()), e = vr(e, t), e.unshift(t);
    },
    remove(t) {
      var n;
      e = vr(e, t), (n = e[0]) == null || n.resume();
    }
  };
}
function vr(e, t) {
  const n = [...e], r = n.indexOf(t);
  return r !== -1 && n.splice(r, 1), n;
}
function Fc(e) {
  return e.filter((t) => t.tagName !== "A");
}
const zc = ["top", "right", "bottom", "left"], Oe = Math.min, Ee = Math.max, Mt = Math.round, Et = Math.floor, ke = (e) => ({
  x: e,
  y: e
}), $c = {
  left: "right",
  right: "left",
  bottom: "top",
  top: "bottom"
};
function Lo(e, t, n) {
  return Ee(e, Oe(t, n));
}
function Re(e, t) {
  return typeof e == "function" ? e(t) : e;
}
function Me(e) {
  return e.split("-")[0];
}
function rt(e) {
  return e.split("-")[1];
}
function Ln(e) {
  return e === "x" ? "y" : "x";
}
function Dn(e) {
  return e === "y" ? "height" : "width";
}
function we(e) {
  const t = e[0];
  return t === "t" || t === "b" ? "y" : "x";
}
function Fn(e) {
  return Ln(we(e));
}
function Wc(e, t, n) {
  n === void 0 && (n = !1);
  const r = rt(e), o = Fn(e), s = Dn(o);
  let i = o === "x" ? r === (n ? "end" : "start") ? "right" : "left" : r === "start" ? "bottom" : "top";
  return t.reference[s] > t.floating[s] && (i = Lt(i)), [i, Lt(i)];
}
function Bc(e) {
  const t = Lt(e);
  return [yn(e), t, yn(t)];
}
function yn(e) {
  return e.includes("start") ? e.replace("start", "end") : e.replace("end", "start");
}
const br = ["left", "right"], yr = ["right", "left"], Vc = ["top", "bottom"], Hc = ["bottom", "top"];
function Uc(e, t, n) {
  switch (e) {
    case "top":
    case "bottom":
      return n ? t ? yr : br : t ? br : yr;
    case "left":
    case "right":
      return t ? Vc : Hc;
    default:
      return [];
  }
}
function jc(e, t, n, r) {
  const o = rt(e);
  let s = Uc(Me(e), n === "start", r);
  return o && (s = s.map((i) => i + "-" + o), t && (s = s.concat(s.map(yn)))), s;
}
function Lt(e) {
  const t = Me(e);
  return $c[t] + e.slice(t.length);
}
function Gc(e) {
  var t, n, r, o;
  return {
    top: (t = e.top) != null ? t : 0,
    right: (n = e.right) != null ? n : 0,
    bottom: (r = e.bottom) != null ? r : 0,
    left: (o = e.left) != null ? o : 0
  };
}
function Do(e) {
  return typeof e != "number" ? Gc(e) : {
    top: e,
    right: e,
    bottom: e,
    left: e
  };
}
function Dt(e) {
  const {
    x: t,
    y: n,
    width: r,
    height: o
  } = e;
  return {
    width: r,
    height: o,
    top: n,
    left: t,
    right: t + r,
    bottom: n + o,
    x: t,
    y: n
  };
}
function wr(e, t, n) {
  let {
    reference: r,
    floating: o
  } = e;
  const s = we(t), i = Fn(t), a = Dn(i), c = Me(t), l = s === "y", f = r.x + r.width / 2 - o.width / 2, m = r.y + r.height / 2 - o.height / 2, h = r[a] / 2 - o[a] / 2;
  let g;
  switch (c) {
    case "top":
      g = {
        x: f,
        y: r.y - o.height
      };
      break;
    case "bottom":
      g = {
        x: f,
        y: r.y + r.height
      };
      break;
    case "right":
      g = {
        x: r.x + r.width,
        y: m
      };
      break;
    case "left":
      g = {
        x: r.x - o.width,
        y: m
      };
      break;
    default:
      g = {
        x: r.x,
        y: r.y
      };
  }
  const w = rt(t);
  return w && (g[i] += h * (w === "end" ? 1 : -1) * (n && l ? -1 : 1)), g;
}
async function Kc(e, t) {
  var n;
  t === void 0 && (t = {});
  const {
    x: r,
    y: o,
    platform: s,
    rects: i,
    elements: a,
    strategy: c
  } = e, {
    boundary: l = "clippingAncestors",
    rootBoundary: f = "viewport",
    elementContext: m = "floating",
    altBoundary: h = !1,
    padding: g = 0
  } = Re(t, e), w = Do(g), x = a[h ? m === "floating" ? "reference" : "floating" : m], b = Dt(await s.getClippingRect({
    element: (n = await (s.isElement == null ? void 0 : s.isElement(x))) == null || n ? x : x.contextElement || await (s.getDocumentElement == null ? void 0 : s.getDocumentElement(a.floating)),
    boundary: l,
    rootBoundary: f,
    strategy: c
  })), y = m === "floating" ? {
    x: r,
    y: o,
    width: i.floating.width,
    height: i.floating.height
  } : i.reference, v = await (s.getOffsetParent == null ? void 0 : s.getOffsetParent(a.floating)), S = await (s.isElement == null ? void 0 : s.isElement(v)) && await (s.getScale == null ? void 0 : s.getScale(v)) || {
    x: 1,
    y: 1
  }, E = Dt(s.convertOffsetParentRelativeRectToViewportRelativeRect ? await s.convertOffsetParentRelativeRectToViewportRelativeRect({
    elements: a,
    rect: y,
    offsetParent: v,
    strategy: c
  }) : y);
  return {
    top: (b.top - E.top + w.top) / S.y,
    bottom: (E.bottom - b.bottom + w.bottom) / S.y,
    left: (b.left - E.left + w.left) / S.x,
    right: (E.right - b.right + w.right) / S.x
  };
}
const qc = 50, Yc = async (e, t, n) => {
  const {
    placement: r = "bottom",
    strategy: o = "absolute",
    middleware: s = [],
    platform: i
  } = n, a = i.detectOverflow ? i : {
    ...i,
    detectOverflow: Kc
  }, c = await (i.isRTL == null ? void 0 : i.isRTL(t));
  let l = await i.getElementRects({
    reference: e,
    floating: t,
    strategy: o
  }), {
    x: f,
    y: m
  } = wr(l, r, c), h = r, g = 0;
  const w = {};
  for (let p = 0; p < s.length; p++) {
    const x = s[p];
    if (!x)
      continue;
    const {
      name: b,
      fn: y
    } = x, {
      x: v,
      y: S,
      data: E,
      reset: C
    } = await y({
      x: f,
      y: m,
      initialPlacement: r,
      placement: h,
      strategy: o,
      middlewareData: w,
      rects: l,
      platform: a,
      elements: {
        reference: e,
        floating: t
      }
    });
    f = v ?? f, m = S ?? m, w[b] = {
      ...w[b],
      ...E
    }, C && g < qc && (g++, typeof C == "object" && (C.placement && (h = C.placement), C.rects && (l = C.rects === !0 ? await i.getElementRects({
      reference: e,
      floating: t,
      strategy: o
    }) : C.rects), {
      x: f,
      y: m
    } = wr(l, h, c)), p = -1);
  }
  return {
    x: f,
    y: m,
    placement: h,
    strategy: o,
    middlewareData: w
  };
}, Xc = (e) => ({
  name: "arrow",
  options: e,
  async fn(t) {
    const {
      x: n,
      y: r,
      placement: o,
      rects: s,
      platform: i,
      elements: a,
      middlewareData: c
    } = t, {
      element: l,
      padding: f = 0
    } = Re(e, t) || {};
    if (l == null)
      return {};
    const m = Do(f), h = {
      x: n,
      y: r
    }, g = Fn(o), w = Dn(g), p = await i.getDimensions(l), x = g === "y", b = x ? "top" : "left", y = x ? "bottom" : "right", v = x ? "clientHeight" : "clientWidth", S = s.reference[w] + s.reference[g] - h[g] - s.floating[w], E = h[g] - s.reference[g], C = await (i.getOffsetParent == null ? void 0 : i.getOffsetParent(l));
    let k = C ? C[v] : 0;
    (!k || !await (i.isElement == null ? void 0 : i.isElement(C))) && (k = a.floating[v] || s.floating[w]);
    const N = S / 2 - E / 2, L = k / 2 - p[w] / 2 - 1, F = Oe(m[b], L), _ = Oe(m[y], L), B = k - p[w] - _, $ = k / 2 - p[w] / 2 + N, H = Lo(F, $, B), U = !c.arrow && rt(o) != null && $ !== H && s.reference[w] / 2 - ($ < F ? F : _) - p[w] / 2 < 0, O = U ? $ < F ? $ - F : $ - B : 0;
    return {
      [g]: h[g] + O,
      data: {
        [g]: H,
        centerOffset: $ - H - O,
        ...U && {
          alignmentOffset: O
        }
      },
      reset: U
    };
  }
}), Zc = function(e) {
  return e === void 0 && (e = {}), {
    name: "flip",
    options: e,
    async fn(t) {
      var n, r;
      const {
        placement: o,
        middlewareData: s,
        rects: i,
        initialPlacement: a,
        platform: c,
        elements: l
      } = t, {
        mainAxis: f = !0,
        crossAxis: m = !0,
        fallbackPlacements: h,
        fallbackStrategy: g = "bestFit",
        fallbackAxisSideDirection: w = "none",
        flipAlignment: p = !0,
        ...x
      } = Re(e, t);
      if ((n = s.arrow) != null && n.alignmentOffset)
        return {};
      const b = Me(o), y = we(a), v = Me(a) === a, S = await (c.isRTL == null ? void 0 : c.isRTL(l.floating)), E = h || (v || !p ? [Lt(a)] : Bc(a)), C = w !== "none";
      !h && C && E.push(...jc(a, p, w, S));
      const k = [a, ...E], N = await c.detectOverflow(t, x), L = [];
      let F = ((r = s.flip) == null ? void 0 : r.overflows) || [];
      if (f && L.push(N[b]), m) {
        const H = Wc(o, i, S);
        L.push(N[H[0]], N[H[1]]);
      }
      if (F = [...F, {
        placement: o,
        overflows: L
      }], !L.every((H) => H <= 0)) {
        var _, B;
        const H = (((_ = s.flip) == null ? void 0 : _.index) || 0) + 1, U = k[H];
        if (U && (!(m === "alignment" ? y !== we(U) : !1) || // We leave the current main axis only if every placement on that axis
        // overflows the main axis.
        F.every((M) => we(M.placement) === y ? M.overflows[0] > 0 : !0)))
          return {
            data: {
              index: H,
              overflows: F
            },
            reset: {
              placement: U
            }
          };
        let O = (B = F.filter((G) => G.overflows[0] <= 0).sort((G, M) => G.overflows[1] - M.overflows[1])[0]) == null ? void 0 : B.placement;
        if (!O)
          switch (g) {
            case "bestFit": {
              var $;
              const G = ($ = F.filter((M) => {
                if (C) {
                  const K = we(M.placement);
                  return K === y || // Create a bias to the `y` side axis due to horizontal
                  // reading directions favoring greater width.
                  K === "y";
                }
                return !0;
              }).map((M) => [M.placement, M.overflows.filter((K) => K > 0).reduce((K, P) => K + P, 0)]).sort((M, K) => M[1] - K[1])[0]) == null ? void 0 : $[0];
              G && (O = G);
              break;
            }
            case "initialPlacement":
              O = a;
              break;
          }
        if (o !== O)
          return {
            reset: {
              placement: O
            }
          };
      }
      return {};
    }
  };
};
function xr(e, t) {
  return {
    top: e.top - t.height,
    right: e.right - t.width,
    bottom: e.bottom - t.height,
    left: e.left - t.width
  };
}
function Cr(e) {
  return zc.some((t) => e[t] >= 0);
}
const Qc = function(e) {
  return e === void 0 && (e = {}), {
    name: "hide",
    options: e,
    async fn(t) {
      const {
        rects: n,
        platform: r
      } = t, {
        strategy: o = "referenceHidden",
        ...s
      } = Re(e, t);
      switch (o) {
        case "referenceHidden": {
          const i = await r.detectOverflow(t, {
            ...s,
            elementContext: "reference"
          }), a = xr(i, n.reference);
          return {
            data: {
              referenceHiddenOffsets: a,
              referenceHidden: Cr(a)
            }
          };
        }
        case "escaped": {
          const i = await r.detectOverflow(t, {
            ...s,
            altBoundary: !0
          }), a = xr(i, n.floating);
          return {
            data: {
              escapedOffsets: a,
              escaped: Cr(a)
            }
          };
        }
        default:
          return {};
      }
    }
  };
}, Fo = /* @__PURE__ */ new Set(["left", "top"]);
async function Jc(e, t) {
  const {
    placement: n,
    platform: r,
    elements: o
  } = e, s = await (r.isRTL == null ? void 0 : r.isRTL(o.floating)), i = Me(n), a = rt(n), c = we(n) === "y", l = Fo.has(i) ? -1 : 1, f = s && c ? -1 : 1, m = Re(t, e);
  let {
    mainAxis: h,
    crossAxis: g,
    alignmentAxis: w
  } = typeof m == "number" ? {
    mainAxis: m,
    crossAxis: 0,
    alignmentAxis: null
  } : {
    mainAxis: m.mainAxis || 0,
    crossAxis: m.crossAxis || 0,
    alignmentAxis: m.alignmentAxis
  };
  return a && typeof w == "number" && (g = a === "end" ? w * -1 : w), c ? {
    x: g * f,
    y: h * l
  } : {
    x: h * l,
    y: g * f
  };
}
const el = function(e) {
  return e === void 0 && (e = 0), {
    name: "offset",
    options: e,
    async fn(t) {
      var n, r;
      const {
        x: o,
        y: s,
        placement: i,
        middlewareData: a
      } = t, c = await Jc(t, e);
      return i === ((n = a.offset) == null ? void 0 : n.placement) && (r = a.arrow) != null && r.alignmentOffset ? {} : {
        x: o + c.x,
        y: s + c.y,
        data: {
          ...c,
          placement: i
        }
      };
    }
  };
}, tl = function(e) {
  return e === void 0 && (e = {}), {
    name: "shift",
    options: e,
    async fn(t) {
      const {
        x: n,
        y: r,
        placement: o,
        platform: s
      } = t, {
        mainAxis: i = !0,
        crossAxis: a = !1,
        limiter: c = {
          fn: (y) => {
            let {
              x: v,
              y: S
            } = y;
            return {
              x: v,
              y: S
            };
          }
        },
        ...l
      } = Re(e, t), f = {
        x: n,
        y: r
      }, m = await s.detectOverflow(t, l), h = we(o), g = Ln(h);
      let w = f[g], p = f[h];
      const x = (y, v) => Lo(v + m[y === "y" ? "top" : "left"], v, v - m[y === "y" ? "bottom" : "right"]);
      i && (w = x(g, w)), a && (p = x(h, p));
      const b = c.fn({
        ...t,
        [g]: w,
        [h]: p
      });
      return {
        ...b,
        data: {
          x: b.x - n,
          y: b.y - r,
          enabled: {
            [g]: i,
            [h]: a
          }
        }
      };
    }
  };
}, nl = function(e) {
  return e === void 0 && (e = {}), {
    options: e,
    fn(t) {
      var n, r;
      const {
        x: o,
        y: s,
        placement: i,
        rects: a,
        middlewareData: c
      } = t, {
        offset: l = 0,
        mainAxis: f = !0,
        crossAxis: m = !0
      } = Re(e, t), h = {
        x: o,
        y: s
      }, g = we(i), w = Ln(g);
      let p = h[w], x = h[g];
      const b = Re(l, t), y = typeof b == "number" ? {
        mainAxis: b,
        crossAxis: 0
      } : {
        mainAxis: (n = b.mainAxis) != null ? n : 0,
        crossAxis: (r = b.crossAxis) != null ? r : 0
      };
      if (f) {
        const E = w === "y" ? "height" : "width", C = a.reference[w] - a.floating[E] + y.mainAxis, k = a.reference[w] + a.reference[E] - y.mainAxis;
        p < C ? p = C : p > k && (p = k);
      }
      if (m) {
        var v, S;
        const E = w === "y" ? "width" : "height", C = Fo.has(Me(i)), k = a.reference[g] - a.floating[E] + (C && ((v = c.offset) == null ? void 0 : v[g]) || 0) + (C ? 0 : y.crossAxis), N = a.reference[g] + a.reference[E] + (C ? 0 : ((S = c.offset) == null ? void 0 : S[g]) || 0) - (C ? y.crossAxis : 0);
        x < k ? x = k : x > N && (x = N);
      }
      return {
        [w]: p,
        [g]: x
      };
    }
  };
}, rl = function(e) {
  return e === void 0 && (e = {}), {
    name: "size",
    options: e,
    async fn(t) {
      const {
        placement: n,
        rects: r,
        platform: o,
        elements: s
      } = t, {
        apply: i = () => {
        },
        ...a
      } = Re(e, t), c = await o.detectOverflow(t, a), l = Me(n), f = rt(n), m = we(n) === "y", {
        width: h,
        height: g
      } = r.floating;
      let w, p;
      l === "top" || l === "bottom" ? (w = l, p = f === (await (o.isRTL == null ? void 0 : o.isRTL(s.floating)) ? "start" : "end") ? "left" : "right") : (p = l, w = f === "end" ? "top" : "bottom");
      const x = g - c.top - c.bottom, b = h - c.left - c.right, y = Oe(g - c[w], x), v = Oe(h - c[p], b), S = t.middlewareData.shift, E = !S;
      let C = y, k = v;
      S != null && S.enabled.x && (k = b), S != null && S.enabled.y && (C = x), E && !f && (m ? k = h - 2 * Ee(c.left, c.right) : C = g - 2 * Ee(c.top, c.bottom)), await i({
        ...t,
        availableWidth: k,
        availableHeight: C
      });
      const N = await o.getDimensions(s.floating);
      return h !== N.width || g !== N.height ? {
        reset: {
          rects: !0
        }
      } : {};
    }
  };
};
function Ht() {
  return typeof window < "u";
}
function ot(e) {
  return zo(e) ? (e.nodeName || "").toLowerCase() : "#document";
}
function ue(e) {
  var t;
  return (e == null || (t = e.ownerDocument) == null ? void 0 : t.defaultView) || window;
}
function Te(e) {
  var t;
  return (t = (zo(e) ? e.ownerDocument : e.document) || window.document) == null ? void 0 : t.documentElement;
}
function zo(e) {
  return Ht() ? e instanceof Node || e instanceof ue(e).Node : !1;
}
function Ce(e) {
  return Ht() ? e instanceof Element || e instanceof ue(e).Element : !1;
}
function Fe(e) {
  return Ht() ? e instanceof HTMLElement || e instanceof ue(e).HTMLElement : !1;
}
function Sr(e) {
  return !Ht() || typeof ShadowRoot > "u" ? !1 : e instanceof ShadowRoot || e instanceof ue(e).ShadowRoot;
}
function Ut(e) {
  const {
    overflow: t,
    overflowX: n,
    overflowY: r,
    display: o
  } = Se(e);
  return /auto|scroll|overlay|hidden|clip/.test(t + r + n) && o !== "inline" && o !== "contents";
}
function ol(e) {
  return /^(table|td|th)$/.test(ot(e));
}
function jt(e) {
  try {
    if (e.matches(":popover-open"))
      return !0;
  } catch {
  }
  try {
    return e.matches(":modal");
  } catch {
    return !1;
  }
}
const sl = /transform|translate|scale|rotate|perspective|filter/, il = /paint|layout|strict|content/, Be = (e) => !!e && e !== "none";
let cn;
function zn(e) {
  const t = Ce(e) ? Se(e) : e;
  return Be(t.transform) || Be(t.translate) || Be(t.scale) || Be(t.rotate) || Be(t.perspective) || !$n() && (Be(t.backdropFilter) || Be(t.filter)) || sl.test(t.willChange || "") || il.test(t.contain || "");
}
function al(e) {
  let t = Ve(e);
  for (; Fe(t) && !mt(t); ) {
    if (zn(t))
      return t;
    if (jt(t))
      return null;
    t = Ve(t);
  }
  return null;
}
function $n() {
  return cn == null && (cn = typeof CSS < "u" && CSS.supports && CSS.supports("-webkit-backdrop-filter", "none")), cn;
}
function mt(e) {
  return /^(html|body|#document)$/.test(ot(e));
}
function Se(e) {
  return ue(e).getComputedStyle(e);
}
function Gt(e) {
  return Ce(e) ? {
    scrollLeft: e.scrollLeft,
    scrollTop: e.scrollTop
  } : {
    scrollLeft: e.scrollX,
    scrollTop: e.scrollY
  };
}
function Ve(e) {
  if (ot(e) === "html")
    return e;
  const t = (
    // Step into the shadow DOM of the parent of a slotted node.
    e.assignedSlot || // DOM Element detected.
    e.parentNode || // ShadowRoot detected.
    Sr(e) && e.host || // Fallback.
    Te(e)
  );
  return Sr(t) ? t.host : t;
}
function $o(e) {
  const t = Ve(e);
  return mt(t) ? (e.ownerDocument || e).body : Fe(t) && Ut(t) ? t : $o(t);
}
function pt(e, t, n) {
  var r;
  t === void 0 && (t = []), n === void 0 && (n = !0);
  const o = $o(e), s = o === ((r = e.ownerDocument) == null ? void 0 : r.body), i = ue(o);
  if (s) {
    const a = wn(i);
    return t.concat(i, i.visualViewport || [], Ut(o) ? o : [], a && n ? pt(a) : []);
  } else
    return t.concat(o, pt(o, [], n));
}
function wn(e) {
  return e.parent && Object.getPrototypeOf(e.parent) ? e.frameElement : null;
}
function Wo(e) {
  const t = Se(e);
  let n = parseFloat(t.width) || 0, r = parseFloat(t.height) || 0;
  const o = Fe(e), s = o ? e.offsetWidth : n, i = o ? e.offsetHeight : r, a = Mt(n) !== s || Mt(r) !== i;
  return a && (n = s, r = i), {
    width: n,
    height: r,
    $: a
  };
}
function Wn(e) {
  return Ce(e) ? e : e.contextElement;
}
function Ze(e) {
  const t = Wn(e);
  if (!Fe(t))
    return ke(1);
  const n = t.getBoundingClientRect(), {
    width: r,
    height: o,
    $: s
  } = Wo(t);
  let i = (s ? Mt(n.width) : n.width) / r, a = (s ? Mt(n.height) : n.height) / o;
  return (!i || !Number.isFinite(i)) && (i = 1), (!a || !Number.isFinite(a)) && (a = 1), {
    x: i,
    y: a
  };
}
const cl = /* @__PURE__ */ ke(0);
function Bo(e) {
  const t = ue(e);
  return !$n() || !t.visualViewport ? cl : {
    x: t.visualViewport.offsetLeft,
    y: t.visualViewport.offsetTop
  };
}
function ll(e, t, n) {
  return t === void 0 && (t = !1), !!n && t && n === ue(e);
}
function He(e, t, n, r) {
  t === void 0 && (t = !1), n === void 0 && (n = !1);
  const o = e.getBoundingClientRect(), s = Wn(e);
  let i = ke(1);
  t && (r ? Ce(r) && (i = Ze(r)) : i = Ze(e));
  const a = ll(s, n, r) ? Bo(s) : ke(0);
  let c = (o.left + a.x) / i.x, l = (o.top + a.y) / i.y, f = o.width / i.x, m = o.height / i.y;
  if (s && r) {
    const h = ue(s), g = Ce(r) ? ue(r) : r;
    let w = h, p = wn(w);
    for (; p && g !== w; ) {
      const x = Ze(p), b = p.getBoundingClientRect(), y = Se(p), v = b.left + (p.clientLeft + parseFloat(y.paddingLeft)) * x.x, S = b.top + (p.clientTop + parseFloat(y.paddingTop)) * x.y;
      c *= x.x, l *= x.y, f *= x.x, m *= x.y, c += v, l += S, w = ue(p), p = wn(w);
    }
  }
  return Dt({
    width: f,
    height: m,
    x: c,
    y: l
  });
}
function Kt(e, t) {
  const n = Gt(e).scrollLeft;
  return t ? t.left + n : He(Te(e)).left + n;
}
function Vo(e, t) {
  const n = e.getBoundingClientRect(), r = n.left + t.scrollLeft - Kt(e, n), o = n.top + t.scrollTop;
  return {
    x: r,
    y: o
  };
}
function dl(e) {
  let {
    elements: t,
    rect: n,
    offsetParent: r,
    strategy: o
  } = e;
  const s = o === "fixed", i = Te(r), a = t ? jt(t.floating) : !1;
  if (r === i || a && s)
    return n;
  let c = {
    scrollLeft: 0,
    scrollTop: 0
  }, l = ke(1);
  const f = ke(0), m = Fe(r);
  if ((m || !s) && ((ot(r) !== "body" || Ut(i)) && (c = Gt(r)), m)) {
    const g = He(r);
    l = Ze(r), f.x = g.x + r.clientLeft, f.y = g.y + r.clientTop;
  }
  const h = i && !m && !s ? Vo(i, c) : ke(0);
  return {
    width: n.width * l.x,
    height: n.height * l.y,
    x: n.x * l.x - c.scrollLeft * l.x + f.x + h.x,
    y: n.y * l.y - c.scrollTop * l.y + f.y + h.y
  };
}
function ul(e) {
  return e.getClientRects ? Array.from(e.getClientRects()) : [];
}
function fl(e) {
  const t = Gt(e), n = e.ownerDocument.body, r = Ee(e.scrollWidth, e.clientWidth, n.scrollWidth, n.clientWidth), o = Ee(e.scrollHeight, e.clientHeight, n.scrollHeight, n.clientHeight);
  let s = -t.scrollLeft + Kt(e);
  const i = -t.scrollTop;
  return Se(n).direction === "rtl" && (s += Ee(e.clientWidth, n.clientWidth) - r), {
    width: r,
    height: o,
    x: s,
    y: i
  };
}
const ml = 25;
function pl(e, t, n) {
  n === void 0 && (n = "viewport");
  const r = n === "layoutViewport", o = ue(e), s = Te(e), i = o.visualViewport;
  let a = s.clientWidth, c = s.clientHeight, l = 0, f = 0;
  if (i) {
    const h = !$n() || t === "fixed";
    r ? h || (l = -i.offsetLeft, f = -i.offsetTop) : (a = i.width, c = i.height, h && (l = i.offsetLeft, f = i.offsetTop));
  }
  if (Kt(s) <= 0) {
    const h = s.ownerDocument, g = h.body, w = getComputedStyle(g), p = h.compatMode === "CSS1Compat" && parseFloat(w.marginLeft) + parseFloat(w.marginRight) || 0, x = Math.abs(s.clientWidth - g.clientWidth - p), b = getComputedStyle(s).scrollbarGutter === "stable both-edges" ? x / 2 : x;
    b <= ml && (a -= b);
  }
  return {
    width: a,
    height: c,
    x: l,
    y: f
  };
}
function hl(e, t) {
  const n = He(e, !0, t === "fixed"), r = n.top + e.clientTop, o = n.left + e.clientLeft, s = Ze(e), i = e.clientWidth * s.x, a = e.clientHeight * s.y, c = o * s.x, l = r * s.y;
  return {
    width: i,
    height: a,
    x: c,
    y: l
  };
}
function Nr(e, t, n) {
  let r;
  if (t === "viewport" || t === "layoutViewport")
    r = pl(e, n, t);
  else if (t === "document")
    r = fl(Te(e));
  else if (Ce(t))
    r = hl(t, n);
  else {
    const o = Bo(e);
    r = {
      x: t.x - o.x,
      y: t.y - o.y,
      width: t.width,
      height: t.height
    };
  }
  return Dt(r);
}
function gl(e, t) {
  const n = t.get(e);
  if (n)
    return n;
  let r = pt(e, [], !1).filter((a) => Ce(a) && ot(a) !== "body"), o = null;
  const s = Se(e).position === "fixed";
  let i = s ? Ve(e) : e;
  for (; Ce(i) && !mt(i); ) {
    const a = Se(i), c = zn(i), l = o ? o.position : s ? "fixed" : "";
    !c && (l === "fixed" || l === "absolute" && a.position === "static") ? r = r.filter((m) => m !== i) : o = a, i = Ve(i);
  }
  return t.set(e, r), r;
}
function vl(e) {
  let {
    element: t,
    boundary: n,
    rootBoundary: r,
    strategy: o
  } = e;
  const i = [...n === "clippingAncestors" ? jt(t) ? [] : gl(t, this._c) : [].concat(n), r], a = Nr(t, i[0], o);
  let c = a.top, l = a.right, f = a.bottom, m = a.left;
  for (let h = 1; h < i.length; h++) {
    const g = Nr(t, i[h], o);
    c = Ee(g.top, c), l = Oe(g.right, l), f = Oe(g.bottom, f), m = Ee(g.left, m);
  }
  return {
    width: l - m,
    height: f - c,
    x: m,
    y: c
  };
}
function bl(e) {
  const {
    width: t,
    height: n
  } = Wo(e);
  return {
    width: t,
    height: n
  };
}
function yl(e, t, n) {
  const r = Fe(t), o = Te(t), s = n === "fixed", i = He(e, !0, s, t);
  let a = {
    scrollLeft: 0,
    scrollTop: 0
  };
  const c = ke(0);
  if ((r || !s) && ((ot(t) !== "body" || Ut(o)) && (a = Gt(t)), r)) {
    const h = He(t, !0, s, t);
    c.x = h.x + t.clientLeft, c.y = h.y + t.clientTop;
  }
  !r && o && (c.x = Kt(o));
  const l = o && !r && !s ? Vo(o, a) : ke(0), f = i.left + a.scrollLeft - c.x - l.x, m = i.top + a.scrollTop - c.y - l.y;
  return {
    x: f,
    y: m,
    width: i.width,
    height: i.height
  };
}
function ln(e) {
  return Se(e).position === "static";
}
function Er(e, t) {
  if (!Fe(e) || Se(e).position === "fixed")
    return null;
  if (t)
    return t(e);
  let n = e.offsetParent;
  return Te(e) === n && (n = n.ownerDocument.body), n;
}
function Ho(e, t) {
  const n = ue(e);
  if (jt(e))
    return n;
  if (!Fe(e)) {
    let o = Ve(e);
    for (; o && !mt(o); ) {
      if (Ce(o) && !ln(o))
        return o;
      o = Ve(o);
    }
    return n;
  }
  let r = Er(e, t);
  for (; r && ol(r) && ln(r); )
    r = Er(r, t);
  return r && mt(r) && ln(r) && !zn(r) ? n : r || al(e) || n;
}
const wl = async function(e) {
  const t = this.getOffsetParent || Ho, n = this.getDimensions, r = await n(e.floating);
  return {
    reference: yl(e.reference, await t(e.floating), e.strategy),
    floating: {
      x: 0,
      y: 0,
      width: r.width,
      height: r.height
    }
  };
};
function xl(e) {
  return Se(e).direction === "rtl";
}
const Cl = {
  convertOffsetParentRelativeRectToViewportRelativeRect: dl,
  getDocumentElement: Te,
  getClippingRect: vl,
  getOffsetParent: Ho,
  getElementRects: wl,
  getClientRects: ul,
  getDimensions: bl,
  getScale: Ze,
  isElement: Ce,
  isRTL: xl
};
function Uo(e, t) {
  return e.x === t.x && e.y === t.y && e.width === t.width && e.height === t.height;
}
function Sl(e, t, n) {
  let r = null, o;
  const s = Te(e);
  function i() {
    var f;
    clearTimeout(o), (f = r) == null || f.disconnect(), r = null;
  }
  function a(f, m) {
    f === void 0 && (f = !1), m === void 0 && (m = 1), i();
    const h = e.getBoundingClientRect(), {
      left: g,
      top: w,
      width: p,
      height: x
    } = h;
    if (f || t(), !p || !x)
      return;
    const b = Et(w), y = Et(s.clientWidth - (g + p)), v = Et(s.clientHeight - (w + x)), S = Et(g), C = {
      rootMargin: -b + "px " + -y + "px " + -v + "px " + -S + "px",
      threshold: Ee(0, Oe(1, m)) || 1
    };
    let k = !0;
    function N(L) {
      const F = L[0].intersectionRatio;
      if (!Uo(h, e.getBoundingClientRect()))
        return a();
      if (F !== m) {
        if (!k)
          return a();
        F ? a(!1, F) : o = setTimeout(() => {
          a(!1, 1e-7);
        }, 1e3);
      }
      k = !1;
    }
    try {
      r = new IntersectionObserver(N, {
        ...C,
        // Handle <iframe>s
        root: s.ownerDocument
      });
    } catch {
      r = new IntersectionObserver(N, C);
    }
    r.observe(e);
  }
  const c = ue(e), l = () => a(n);
  return c.addEventListener("resize", l), a(!0), () => {
    c.removeEventListener("resize", l), i();
  };
}
function Nl(e, t, n, r) {
  r === void 0 && (r = {});
  const {
    ancestorScroll: o = !0,
    ancestorResize: s = !0,
    elementResize: i = typeof ResizeObserver == "function",
    layoutShift: a = typeof IntersectionObserver == "function",
    animationFrame: c = !1
  } = r, l = Wn(e), f = o || s ? [...l ? pt(l) : [], ...t ? pt(t) : []] : [];
  f.forEach((b) => {
    o && b.addEventListener("scroll", n), s && b.addEventListener("resize", n);
  });
  const m = l && a ? Sl(l, n, s) : null;
  let h = -1, g = null;
  i && (g = new ResizeObserver((b) => {
    let [y] = b;
    y && y.target === l && g && t && (g.unobserve(t), cancelAnimationFrame(h), h = requestAnimationFrame(() => {
      var v;
      (v = g) == null || v.observe(t);
    })), n();
  }), l && !c && g.observe(l), t && g.observe(t));
  let w, p = c ? He(e) : null;
  c && x();
  function x() {
    const b = He(e);
    p && !Uo(p, b) && n(), p = b, w = requestAnimationFrame(x);
  }
  return n(), () => {
    var b;
    f.forEach((y) => {
      o && y.removeEventListener("scroll", n), s && y.removeEventListener("resize", n);
    }), m == null || m(), (b = g) == null || b.disconnect(), g = null, c && cancelAnimationFrame(w);
  };
}
const El = el, kl = tl, Rl = Zc, Pl = rl, Al = Qc, kr = Xc, Tl = nl, _l = (e, t, n) => {
  const r = /* @__PURE__ */ new Map(), o = n ?? {}, s = {
    ...Cl,
    ...o.platform,
    _c: r
  };
  return Yc(e, t, {
    ...o,
    platform: s
  });
};
var Il = typeof document < "u", Ol = function() {
}, Tt = Il ? oi : Ol;
function Ft(e, t) {
  if (e === t)
    return !0;
  if (typeof e != typeof t)
    return !1;
  if (typeof e == "function" && e.toString() === t.toString())
    return !0;
  let n, r, o;
  if (e && t && typeof e == "object") {
    if (Array.isArray(e)) {
      if (n = e.length, n !== t.length) return !1;
      for (r = n; r-- !== 0; )
        if (!Ft(e[r], t[r]))
          return !1;
      return !0;
    }
    if (o = Object.keys(e), n = o.length, n !== Object.keys(t).length)
      return !1;
    for (r = n; r-- !== 0; )
      if (!{}.hasOwnProperty.call(t, o[r]))
        return !1;
    for (r = n; r-- !== 0; ) {
      const s = o[r];
      if (!(s === "_owner" && e.$$typeof) && !Ft(e[s], t[s]))
        return !1;
    }
    return !0;
  }
  return e !== e && t !== t;
}
function jo(e) {
  return typeof window > "u" ? 1 : (e.ownerDocument.defaultView || window).devicePixelRatio || 1;
}
function Rr(e, t) {
  const n = jo(e);
  return Math.round(t * n) / n;
}
function dn(e) {
  const t = u.useRef(e);
  return Tt(() => {
    t.current = e;
  }), t;
}
function Ml(e) {
  e === void 0 && (e = {});
  const {
    placement: t = "bottom",
    strategy: n = "absolute",
    middleware: r = [],
    platform: o,
    elements: {
      reference: s,
      floating: i
    } = {},
    transform: a = !0,
    whileElementsMounted: c,
    open: l
  } = e, [f, m] = u.useState({
    x: 0,
    y: 0,
    strategy: n,
    placement: t,
    middlewareData: {},
    isPositioned: !1
  }), [h, g] = u.useState(r);
  Ft(h, r) || g(r);
  const [w, p] = u.useState(null), [x, b] = u.useState(null), y = u.useCallback((M) => {
    M !== C.current && (C.current = M, p(M));
  }, []), v = u.useCallback((M) => {
    M !== k.current && (k.current = M, b(M));
  }, []), S = s || w, E = i || x, C = u.useRef(null), k = u.useRef(null), N = u.useRef(f), L = c != null, F = dn(c), _ = dn(o), B = dn(l), $ = u.useCallback(() => {
    if (!C.current || !k.current)
      return;
    const M = {
      placement: t,
      strategy: n,
      middleware: h
    };
    _.current && (M.platform = _.current), _l(C.current, k.current, M).then((K) => {
      const P = {
        ...K,
        // The floating element's position may be recomputed while it's closed
        // but still mounted (such as when transitioning out). To ensure
        // `isPositioned` will be `false` initially on the next open, avoid
        // setting it to `true` when `open === false` (must be specified).
        isPositioned: B.current !== !1
      };
      H.current && !Ft(N.current, P) && (N.current = P, vt.flushSync(() => {
        m(P);
      }));
    });
  }, [h, t, n, _, B]);
  Tt(() => {
    l === !1 && N.current.isPositioned && (N.current.isPositioned = !1, m((M) => ({
      ...M,
      isPositioned: !1
    })));
  }, [l]);
  const H = u.useRef(!1);
  Tt(() => (H.current = !0, () => {
    H.current = !1;
  }), []), Tt(() => {
    if (S && (C.current = S), E && (k.current = E), S && E) {
      if (F.current)
        return F.current(S, E, $);
      $();
    }
  }, [S, E, $, F, L]);
  const U = u.useMemo(() => ({
    reference: C,
    floating: k,
    setReference: y,
    setFloating: v
  }), [y, v]), O = u.useMemo(() => ({
    reference: S,
    floating: E
  }), [S, E]), G = u.useMemo(() => {
    const M = {
      position: n,
      left: 0,
      top: 0
    };
    if (!O.floating)
      return M;
    const K = Rr(O.floating, f.x), P = Rr(O.floating, f.y);
    return a ? {
      ...M,
      transform: "translate(" + K + "px, " + P + "px)",
      ...jo(O.floating) >= 1.5 && {
        willChange: "transform"
      }
    } : {
      position: n,
      left: K,
      top: P
    };
  }, [n, a, O.floating, f.x, f.y]);
  return u.useMemo(() => ({
    ...f,
    update: $,
    refs: U,
    elements: O,
    floatingStyles: G
  }), [f, $, U, O, G]);
}
const Ll = (e) => {
  function t(n) {
    return {}.hasOwnProperty.call(n, "current");
  }
  return {
    name: "arrow",
    options: e,
    fn(n) {
      const {
        element: r,
        padding: o
      } = typeof e == "function" ? e(n) : e;
      return r && t(r) ? r.current != null ? kr({
        element: r.current,
        padding: o
      }).fn(n) : {} : r ? kr({
        element: r,
        padding: o
      }).fn(n) : {};
    }
  };
}, Dl = (e, t) => {
  const n = El(e);
  return {
    name: n.name,
    fn: n.fn,
    options: [e, t]
  };
}, Fl = (e, t) => {
  const n = kl(e);
  return {
    name: n.name,
    fn: n.fn,
    options: [e, t]
  };
}, zl = (e, t) => ({
  fn: Tl(e).fn,
  options: [e, t]
}), $l = (e, t) => {
  const n = Rl(e);
  return {
    name: n.name,
    fn: n.fn,
    options: [e, t]
  };
}, Wl = (e, t) => {
  const n = Pl(e);
  return {
    name: n.name,
    fn: n.fn,
    options: [e, t]
  };
}, Bl = (e, t) => {
  const n = Al(e);
  return {
    name: n.name,
    fn: n.fn,
    options: [e, t]
  };
}, Vl = (e, t) => {
  const n = Ll(e);
  return {
    name: n.name,
    fn: n.fn,
    options: [e, t]
  };
};
var Hl = "Arrow", Go = u.forwardRef((e, t) => {
  const { children: n, width: r = 10, height: o = 5, ...s } = e;
  return /* @__PURE__ */ d(
    q.svg,
    {
      ...s,
      ref: t,
      width: r,
      height: o,
      viewBox: "0 0 30 10",
      preserveAspectRatio: "none",
      children: e.asChild ? n : /* @__PURE__ */ d("polygon", { points: "0,0 30,0 15,10" })
    }
  );
});
Go.displayName = Hl;
var Ul = Go;
function Ko(e) {
  const [t, n] = u.useState(void 0);
  return se(() => {
    if (e) {
      n({ width: e.offsetWidth, height: e.offsetHeight });
      const r = new ResizeObserver((o) => {
        if (!Array.isArray(o) || !o.length)
          return;
        const s = o[0];
        let i, a;
        if ("borderBoxSize" in s) {
          const c = s.borderBoxSize, l = Array.isArray(c) ? c[0] : c;
          i = l.inlineSize, a = l.blockSize;
        } else
          i = e.offsetWidth, a = e.offsetHeight;
        n({ width: i, height: a });
      });
      return r.observe(e, { box: "border-box" }), () => r.unobserve(e);
    } else
      n(void 0);
  }, [e]), t;
}
var Bn = "Popper", [qo, Yo] = nt(Bn), [jl, Xo] = qo(Bn), Zo = (e) => {
  const { __scopePopper: t, children: n } = e, [r, o] = u.useState(null), [s, i] = u.useState(void 0);
  return /* @__PURE__ */ d(
    jl,
    {
      scope: t,
      anchor: r,
      onAnchorChange: o,
      placementState: s,
      setPlacementState: i,
      children: n
    }
  );
};
Zo.displayName = Bn;
var Qo = "PopperAnchor", Jo = u.forwardRef(
  (e, t) => {
    const { __scopePopper: n, virtualRef: r, ...o } = e, s = Xo(Qo, n), i = u.useRef(null), a = s.onAnchorChange, c = u.useCallback(
      (w) => {
        i.current = w, w && a(w);
      },
      [a]
    ), l = re(t, c), f = u.useRef(null);
    u.useEffect(() => {
      if (!r)
        return;
      const w = f.current;
      f.current = r.current, w !== f.current && a(f.current);
    });
    const m = s.placementState && Hn(s.placementState), h = m == null ? void 0 : m[0], g = m == null ? void 0 : m[1];
    return r ? null : /* @__PURE__ */ d(
      q.div,
      {
        "data-radix-popper-side": h,
        "data-radix-popper-align": g,
        ...o,
        ref: l
      }
    );
  }
);
Jo.displayName = Qo;
var Vn = "PopperContent", [Gl, Kl] = qo(Vn), es = u.forwardRef(
  (e, t) => {
    var Z, V, X, W, j, de;
    const {
      __scopePopper: n,
      side: r = "bottom",
      sideOffset: o = 0,
      align: s = "center",
      alignOffset: i = 0,
      arrowPadding: a = 0,
      avoidCollisions: c = !0,
      collisionBoundary: l = [],
      collisionPadding: f = 0,
      sticky: m = "partial",
      hideWhenDetached: h = !1,
      updatePositionStrategy: g = "optimized",
      onPlaced: w,
      ...p
    } = e, x = Xo(Vn, n), [b, y] = u.useState(null), v = re(t, y), [S, E] = u.useState(null), C = Ko(S), k = (C == null ? void 0 : C.width) ?? 0, N = (C == null ? void 0 : C.height) ?? 0, L = r + (s !== "center" ? "-" + s : ""), F = typeof f == "number" ? f : { top: 0, right: 0, bottom: 0, left: 0, ...f }, _ = Array.isArray(l) ? l : [l], B = _.length > 0, $ = {
      padding: F,
      boundary: _.filter(Yl),
      // with `strategy: 'fixed'`, this is the only way to get it to respect boundaries
      altBoundary: B
    }, { refs: H, floatingStyles: U, placement: O, isPositioned: G, middlewareData: M } = Ml({
      // default to `fixed` strategy so users don't have to pick and we also avoid focus scroll issues
      strategy: "fixed",
      placement: L,
      whileElementsMounted: (...ae) => Nl(...ae, {
        animationFrame: g === "always"
      }),
      elements: {
        reference: x.anchor
      },
      middleware: [
        Dl({ mainAxis: o + N, alignmentAxis: i }),
        c && Fl({
          mainAxis: !0,
          crossAxis: !1,
          limiter: m === "partial" ? zl() : void 0,
          ...$
        }),
        c && $l({ ...$ }),
        Wl({
          ...$,
          apply: ({ elements: ae, rects: Ke, availableWidth: st, availableHeight: it }) => {
            const { width: ei, height: ti } = Ke.reference, wt = ae.floating.style;
            wt.setProperty("--radix-popper-available-width", `${st}px`), wt.setProperty("--radix-popper-available-height", `${it}px`), wt.setProperty("--radix-popper-anchor-width", `${ei}px`), wt.setProperty("--radix-popper-anchor-height", `${ti}px`);
          }
        }),
        S && Vl({ element: S, padding: a }),
        Xl({ arrowWidth: k, arrowHeight: N }),
        h && Bl({
          strategy: "referenceHidden",
          ...$,
          // `hide` detects whether the anchor (reference) is clipped, so when
          // no explicit `collisionBoundary` is set we fall back to Floating
          // UI's default clipping ancestors (e.g. a scrollable menu). This
          // lets an occluded submenu hide once its anchor scrolls out of view
          // (#3237). The collision/size middlewares deliberately keep the
          // viewport-based default to avoid clamping content rendered inside
          // transformed or overflow-clipping portal containers.
          boundary: B ? $.boundary : void 0
        })
      ]
    }), K = x.setPlacementState;
    se(() => (K(O), () => {
      K(void 0);
    }), [O, K]);
    const [P, me] = Hn(O), ee = xe(w);
    se(() => {
      G && (ee == null || ee());
    }, [G, ee]);
    const ce = (Z = M.arrow) == null ? void 0 : Z.x, le = (V = M.arrow) == null ? void 0 : V.y, J = ((X = M.arrow) == null ? void 0 : X.centerOffset) !== 0, [Q, D] = u.useState();
    return se(() => {
      b && D(window.getComputedStyle(b).zIndex);
    }, [b]), /* @__PURE__ */ d(
      "div",
      {
        ref: H.setFloating,
        "data-radix-popper-content-wrapper": "",
        style: {
          ...U,
          transform: G ? U.transform : "translate(0, -200%)",
          // keep off the page when measuring
          minWidth: "max-content",
          zIndex: Q,
          "--radix-popper-transform-origin": [
            (W = M.transformOrigin) == null ? void 0 : W.x,
            (j = M.transformOrigin) == null ? void 0 : j.y
          ].join(" "),
          // hide the content if using the hide middleware and should be hidden
          // set visibility to hidden and disable pointer events so the UI behaves
          // as if the PopperContent isn't there at all
          ...((de = M.hide) == null ? void 0 : de.referenceHidden) && {
            visibility: "hidden",
            pointerEvents: "none"
          }
        },
        dir: e.dir,
        children: /* @__PURE__ */ d(
          Gl,
          {
            scope: n,
            placedSide: P,
            placedAlign: me,
            onArrowChange: E,
            arrowX: ce,
            arrowY: le,
            shouldHideArrow: J,
            children: /* @__PURE__ */ d(
              q.div,
              {
                "data-side": P,
                "data-align": me,
                ...p,
                ref: v,
                style: {
                  ...p.style,
                  // if the PopperContent hasn't been placed yet (not all measurements done)
                  // we prevent animations so that users's animation don't kick in too early referring wrong sides
                  animation: G ? void 0 : "none"
                }
              }
            )
          }
        )
      }
    );
  }
);
es.displayName = Vn;
var ts = "PopperArrow", ql = {
  top: "bottom",
  right: "left",
  bottom: "top",
  left: "right"
}, ns = u.forwardRef(function(t, n) {
  const { __scopePopper: r, ...o } = t, s = Kl(ts, r), i = ql[s.placedSide];
  return (
    // we have to use an extra wrapper because `ResizeObserver` (used by `useSize`)
    // doesn't report size as we'd expect on SVG elements.
    // it reports their bounding box which is effectively the largest path inside the SVG.
    /* @__PURE__ */ d(
      "span",
      {
        ref: s.onArrowChange,
        style: {
          position: "absolute",
          left: s.arrowX,
          top: s.arrowY,
          [i]: 0,
          transformOrigin: {
            top: "",
            right: "0 0",
            bottom: "center 0",
            left: "100% 0"
          }[s.placedSide],
          transform: {
            top: "translateY(100%)",
            right: "translateY(50%) rotate(90deg) translateX(-50%)",
            bottom: "rotate(180deg)",
            left: "translateY(50%) rotate(-90deg) translateX(50%)"
          }[s.placedSide],
          visibility: s.shouldHideArrow ? "hidden" : void 0
        },
        children: /* @__PURE__ */ d(
          Ul,
          {
            ...o,
            ref: n,
            style: {
              ...o.style,
              // ensures the element can be measured correctly (mostly for if SVG)
              display: "block"
            }
          }
        )
      }
    )
  );
});
ns.displayName = ts;
function Yl(e) {
  return e !== null;
}
var Xl = (e) => ({
  name: "transformOrigin",
  options: e,
  fn(t) {
    var x, b, y;
    const { placement: n, rects: r, middlewareData: o } = t, i = ((x = o.arrow) == null ? void 0 : x.centerOffset) !== 0, a = i ? 0 : e.arrowWidth, c = i ? 0 : e.arrowHeight, [l, f] = Hn(n), m = { start: "0%", center: "50%", end: "100%" }[f], h = (((b = o.arrow) == null ? void 0 : b.x) ?? 0) + a / 2, g = (((y = o.arrow) == null ? void 0 : y.y) ?? 0) + c / 2;
    let w = "", p = "";
    return l === "bottom" ? (w = i ? m : `${h}px`, p = `${-c}px`) : l === "top" ? (w = i ? m : `${h}px`, p = `${r.floating.height + c}px`) : l === "right" ? (w = `${-c}px`, p = i ? m : `${g}px`) : l === "left" && (w = `${r.floating.width + c}px`, p = i ? m : `${g}px`), { data: { x: w, y: p } };
  }
});
function Hn(e) {
  const [t, n = "center"] = e.split("-");
  return [t, n];
}
var Zl = Zo, Ql = Jo, Jl = es, ed = ns, td = "Portal", rs = u.forwardRef((e, t) => {
  var a;
  const { container: n, ...r } = e, [o, s] = u.useState(!1);
  se(() => s(!0), []);
  const i = n || o && ((a = globalThis == null ? void 0 : globalThis.document) == null ? void 0 : a.body);
  return i ? vt.createPortal(/* @__PURE__ */ d(q.div, { ...r, ref: t }), i) : null;
});
rs.displayName = td;
function os(e) {
  const t = u.useRef({ value: e, previous: e });
  return u.useMemo(() => (t.current.value !== e && (t.current.previous = t.current.value, t.current.value = e), t.current.previous), [e]);
}
var ss = Object.freeze({
  // See: https://github.com/twbs/bootstrap/blob/main/scss/mixins/_visually-hidden.scss
  position: "absolute",
  border: 0,
  width: 1,
  height: 1,
  padding: 0,
  margin: -1,
  overflow: "hidden",
  clip: "rect(0, 0, 0, 0)",
  whiteSpace: "nowrap",
  wordWrap: "normal"
}), nd = "VisuallyHidden", rd = u.forwardRef(
  (e, t) => /* @__PURE__ */ d(
    q.span,
    {
      ...e,
      ref: t,
      style: { ...ss, ...e.style }
    }
  )
);
rd.displayName = nd;
var od = function(e) {
  if (typeof document > "u")
    return null;
  var t = Array.isArray(e) ? e[0] : e;
  return t.ownerDocument.body;
}, qe = /* @__PURE__ */ new WeakMap(), kt = /* @__PURE__ */ new WeakMap(), Rt = {}, un = 0, is = function(e) {
  return e && (e.host || is(e.parentNode));
}, sd = function(e, t) {
  return t.map(function(n) {
    if (e.contains(n))
      return n;
    var r = is(n);
    return r && e.contains(r) ? r : (console.error("aria-hidden", n, "in not contained inside", e, ". Doing nothing"), null);
  }).filter(function(n) {
    return !!n;
  });
}, id = function(e, t, n, r) {
  var o = sd(t, Array.isArray(e) ? e : [e]);
  Rt[n] || (Rt[n] = /* @__PURE__ */ new WeakMap());
  var s = Rt[n], i = [], a = /* @__PURE__ */ new Set(), c = new Set(o), l = function(m) {
    !m || a.has(m) || (a.add(m), l(m.parentNode));
  };
  o.forEach(l);
  var f = function(m) {
    !m || c.has(m) || Array.prototype.forEach.call(m.children, function(h) {
      if (a.has(h))
        f(h);
      else
        try {
          var g = h.getAttribute(r), w = g !== null && g !== "false", p = (qe.get(h) || 0) + 1, x = (s.get(h) || 0) + 1;
          qe.set(h, p), s.set(h, x), i.push(h), p === 1 && w && kt.set(h, !0), x === 1 && h.setAttribute(n, "true"), w || h.setAttribute(r, "true");
        } catch (b) {
          console.error("aria-hidden: cannot operate on ", h, b);
        }
    });
  };
  return f(t), a.clear(), un++, function() {
    i.forEach(function(m) {
      var h = qe.get(m) - 1, g = s.get(m) - 1;
      qe.set(m, h), s.set(m, g), h || (kt.has(m) || m.removeAttribute(r), kt.delete(m)), g || m.removeAttribute(n);
    }), un--, un || (qe = /* @__PURE__ */ new WeakMap(), qe = /* @__PURE__ */ new WeakMap(), kt = /* @__PURE__ */ new WeakMap(), Rt = {});
  };
}, ad = function(e, t, n) {
  n === void 0 && (n = "data-aria-hidden");
  var r = Array.from(Array.isArray(e) ? e : [e]), o = od(e);
  return o ? (r.push.apply(r, Array.from(o.querySelectorAll("[aria-live], script"))), id(r, o, n, "aria-hidden")) : function() {
    return null;
  };
}, ye = function() {
  return ye = Object.assign || function(t) {
    for (var n, r = 1, o = arguments.length; r < o; r++) {
      n = arguments[r];
      for (var s in n) Object.prototype.hasOwnProperty.call(n, s) && (t[s] = n[s]);
    }
    return t;
  }, ye.apply(this, arguments);
};
function as(e, t) {
  var n = {};
  for (var r in e) Object.prototype.hasOwnProperty.call(e, r) && t.indexOf(r) < 0 && (n[r] = e[r]);
  if (e != null && typeof Object.getOwnPropertySymbols == "function")
    for (var o = 0, r = Object.getOwnPropertySymbols(e); o < r.length; o++)
      t.indexOf(r[o]) < 0 && Object.prototype.propertyIsEnumerable.call(e, r[o]) && (n[r[o]] = e[r[o]]);
  return n;
}
function cd(e, t, n) {
  if (n || arguments.length === 2) for (var r = 0, o = t.length, s; r < o; r++)
    (s || !(r in t)) && (s || (s = Array.prototype.slice.call(t, 0, r)), s[r] = t[r]);
  return e.concat(s || Array.prototype.slice.call(t));
}
var _t = "right-scroll-bar-position", It = "width-before-scroll-bar", ld = "with-scroll-bars-hidden", dd = "--removed-body-scroll-bar-size";
function fn(e, t) {
  return typeof e == "function" ? e(t) : e && (e.current = t), e;
}
function ud(e, t) {
  var n = I(function() {
    return {
      // value
      value: e,
      // last callback
      callback: t,
      // "memoized" public interface
      facade: {
        get current() {
          return n.value;
        },
        set current(r) {
          var o = n.value;
          o !== r && (n.value = r, n.callback(r, o));
        }
      }
    };
  })[0];
  return n.callback = t, n.facade;
}
var fd = typeof window < "u" ? u.useLayoutEffect : u.useEffect, Pr = /* @__PURE__ */ new WeakMap();
function md(e, t) {
  var n = ud(null, function(r) {
    return e.forEach(function(o) {
      return fn(o, r);
    });
  });
  return fd(function() {
    var r = Pr.get(n);
    if (r) {
      var o = new Set(r), s = new Set(e), i = n.current;
      o.forEach(function(a) {
        s.has(a) || fn(a, null);
      }), s.forEach(function(a) {
        o.has(a) || fn(a, i);
      });
    }
    Pr.set(n, e);
  }, [e]), n;
}
function pd(e) {
  return e;
}
function hd(e, t) {
  t === void 0 && (t = pd);
  var n = [], r = !1, o = {
    read: function() {
      if (r)
        throw new Error("Sidecar: could not `read` from an `assigned` medium. `read` could be used only with `useMedium`.");
      return n.length ? n[n.length - 1] : e;
    },
    useMedium: function(s) {
      var i = t(s, r);
      return n.push(i), function() {
        n = n.filter(function(a) {
          return a !== i;
        });
      };
    },
    assignSyncMedium: function(s) {
      for (r = !0; n.length; ) {
        var i = n;
        n = [], i.forEach(s);
      }
      n = {
        push: function(a) {
          return s(a);
        },
        filter: function() {
          return n;
        }
      };
    },
    assignMedium: function(s) {
      r = !0;
      var i = [];
      if (n.length) {
        var a = n;
        n = [], a.forEach(s), i = n;
      }
      var c = function() {
        var f = i;
        i = [], f.forEach(s);
      }, l = function() {
        return Promise.resolve().then(c);
      };
      l(), n = {
        push: function(f) {
          i.push(f), l();
        },
        filter: function(f) {
          return i = i.filter(f), n;
        }
      };
    }
  };
  return o;
}
function gd(e) {
  e === void 0 && (e = {});
  var t = hd(null);
  return t.options = ye({ async: !0, ssr: !1 }, e), t;
}
var cs = function(e) {
  var t = e.sideCar, n = as(e, ["sideCar"]);
  if (!t)
    throw new Error("Sidecar: please provide `sideCar` property to import the right car");
  var r = t.read();
  if (!r)
    throw new Error("Sidecar medium not found");
  return u.createElement(r, ye({}, n));
};
cs.isSideCarExport = !0;
function vd(e, t) {
  return e.useMedium(t), cs;
}
var ls = gd(), mn = function() {
}, qt = u.forwardRef(function(e, t) {
  var n = u.useRef(null), r = u.useState({
    onScrollCapture: mn,
    onWheelCapture: mn,
    onTouchMoveCapture: mn
  }), o = r[0], s = r[1], i = e.forwardProps, a = e.children, c = e.className, l = e.removeScrollBar, f = e.enabled, m = e.shards, h = e.sideCar, g = e.noRelative, w = e.noIsolation, p = e.inert, x = e.allowPinchZoom, b = e.as, y = b === void 0 ? "div" : b, v = e.gapMode, S = as(e, ["forwardProps", "children", "className", "removeScrollBar", "enabled", "shards", "sideCar", "noRelative", "noIsolation", "inert", "allowPinchZoom", "as", "gapMode"]), E = h, C = md([n, t]), k = ye(ye({}, S), o);
  return u.createElement(
    u.Fragment,
    null,
    f && u.createElement(E, { sideCar: ls, removeScrollBar: l, shards: m, noRelative: g, noIsolation: w, inert: p, setCallbacks: s, allowPinchZoom: !!x, lockRef: n, gapMode: v }),
    i ? u.cloneElement(u.Children.only(a), ye(ye({}, k), { ref: C })) : u.createElement(y, ye({}, k, { className: c, ref: C }), a)
  );
});
qt.defaultProps = {
  enabled: !0,
  removeScrollBar: !0,
  inert: !1
};
qt.classNames = {
  fullWidth: It,
  zeroRight: _t
};
var bd = function() {
  if (typeof __webpack_nonce__ < "u")
    return __webpack_nonce__;
};
function yd() {
  if (!document)
    return null;
  var e = document.createElement("style");
  e.type = "text/css";
  var t = bd();
  return t && e.setAttribute("nonce", t), e;
}
function wd(e, t) {
  e.styleSheet ? e.styleSheet.cssText = t : e.appendChild(document.createTextNode(t));
}
function xd(e) {
  var t = document.head || document.getElementsByTagName("head")[0];
  t.appendChild(e);
}
var Cd = function() {
  var e = 0, t = null;
  return {
    add: function(n) {
      e == 0 && (t = yd()) && (wd(t, n), xd(t)), e++;
    },
    remove: function() {
      e--, !e && t && (t.parentNode && t.parentNode.removeChild(t), t = null);
    }
  };
}, Sd = function() {
  var e = Cd();
  return function(t, n) {
    u.useEffect(function() {
      return e.add(t), function() {
        e.remove();
      };
    }, [t && n]);
  };
}, ds = function() {
  var e = Sd(), t = function(n) {
    var r = n.styles, o = n.dynamic;
    return e(r, o), null;
  };
  return t;
}, Nd = {
  left: 0,
  top: 0,
  right: 0,
  gap: 0
}, pn = function(e) {
  return parseInt(e || "", 10) || 0;
}, Ed = function(e) {
  var t = window.getComputedStyle(document.body), n = t[e === "padding" ? "paddingLeft" : "marginLeft"], r = t[e === "padding" ? "paddingTop" : "marginTop"], o = t[e === "padding" ? "paddingRight" : "marginRight"];
  return [pn(n), pn(r), pn(o)];
}, kd = function(e) {
  if (e === void 0 && (e = "margin"), typeof window > "u")
    return Nd;
  var t = Ed(e), n = document.documentElement.clientWidth, r = window.innerWidth;
  return {
    left: t[0],
    top: t[1],
    right: t[2],
    gap: Math.max(0, r - n + t[2] - t[0])
  };
}, Rd = ds(), Qe = "data-scroll-locked", Pd = function(e, t, n, r) {
  var o = e.left, s = e.top, i = e.right, a = e.gap;
  return n === void 0 && (n = "margin"), `
  .`.concat(ld, ` {
   overflow: hidden `).concat(r, `;
   padding-right: `).concat(a, "px ").concat(r, `;
  }
  body[`).concat(Qe, `] {
    overflow: hidden `).concat(r, `;
    overscroll-behavior: contain;
    `).concat([
    t && "position: relative ".concat(r, ";"),
    n === "margin" && `
    padding-left: `.concat(o, `px;
    padding-top: `).concat(s, `px;
    padding-right: `).concat(i, `px;
    margin-left:0;
    margin-top:0;
    margin-right: `).concat(a, "px ").concat(r, `;
    `),
    n === "padding" && "padding-right: ".concat(a, "px ").concat(r, ";")
  ].filter(Boolean).join(""), `
  }
  
  .`).concat(_t, ` {
    right: `).concat(a, "px ").concat(r, `;
  }
  
  .`).concat(It, ` {
    margin-right: `).concat(a, "px ").concat(r, `;
  }
  
  .`).concat(_t, " .").concat(_t, ` {
    right: 0 `).concat(r, `;
  }
  
  .`).concat(It, " .").concat(It, ` {
    margin-right: 0 `).concat(r, `;
  }
  
  body[`).concat(Qe, `] {
    `).concat(dd, ": ").concat(a, `px;
  }
`);
}, Ar = function() {
  var e = parseInt(document.body.getAttribute(Qe) || "0", 10);
  return isFinite(e) ? e : 0;
}, Ad = function() {
  u.useEffect(function() {
    return document.body.setAttribute(Qe, (Ar() + 1).toString()), function() {
      var e = Ar() - 1;
      e <= 0 ? document.body.removeAttribute(Qe) : document.body.setAttribute(Qe, e.toString());
    };
  }, []);
}, Td = function(e) {
  var t = e.noRelative, n = e.noImportant, r = e.gapMode, o = r === void 0 ? "margin" : r;
  Ad();
  var s = u.useMemo(function() {
    return kd(o);
  }, [o]);
  return u.createElement(Rd, { styles: Pd(s, !t, o, n ? "" : "!important") });
}, xn = !1;
if (typeof window < "u")
  try {
    var Pt = Object.defineProperty({}, "passive", {
      get: function() {
        return xn = !0, !0;
      }
    });
    window.addEventListener("test", Pt, Pt), window.removeEventListener("test", Pt, Pt);
  } catch {
    xn = !1;
  }
var Ye = xn ? { passive: !1 } : !1, _d = function(e) {
  return e.tagName === "TEXTAREA";
}, us = function(e, t) {
  if (!(e instanceof Element))
    return !1;
  var n = window.getComputedStyle(e);
  return (
    // not-not-scrollable
    n[t] !== "hidden" && // contains scroll inside self
    !(n.overflowY === n.overflowX && !_d(e) && n[t] === "visible")
  );
}, Id = function(e) {
  return us(e, "overflowY");
}, Od = function(e) {
  return us(e, "overflowX");
}, Tr = function(e, t) {
  var n = t.ownerDocument, r = t;
  do {
    typeof ShadowRoot < "u" && r instanceof ShadowRoot && (r = r.host);
    var o = fs(e, r);
    if (o) {
      var s = ms(e, r), i = s[1], a = s[2];
      if (i > a)
        return !0;
    }
    r = r.parentNode;
  } while (r && r !== n.body);
  return !1;
}, Md = function(e) {
  var t = e.scrollTop, n = e.scrollHeight, r = e.clientHeight;
  return [
    t,
    n,
    r
  ];
}, Ld = function(e) {
  var t = e.scrollLeft, n = e.scrollWidth, r = e.clientWidth;
  return [
    t,
    n,
    r
  ];
}, fs = function(e, t) {
  return e === "v" ? Id(t) : Od(t);
}, ms = function(e, t) {
  return e === "v" ? Md(t) : Ld(t);
}, Dd = function(e, t) {
  return e === "h" && t === "rtl" ? -1 : 1;
}, Fd = function(e, t, n, r, o) {
  var s = Dd(e, window.getComputedStyle(t).direction), i = s * r, a = n.target, c = t.contains(a), l = !1, f = i > 0, m = 0, h = 0;
  do {
    if (!a)
      break;
    var g = ms(e, a), w = g[0], p = g[1], x = g[2], b = p - x - s * w;
    (w || b) && fs(e, a) && (m += b, h += w);
    var y = a.parentNode;
    a = y && y.nodeType === Node.DOCUMENT_FRAGMENT_NODE ? y.host : y;
  } while (
    // portaled content
    !c && a !== document.body || // self content
    c && (t.contains(a) || t === a)
  );
  return (f && Math.abs(m) < 1 || !f && Math.abs(h) < 1) && (l = !0), l;
}, At = function(e) {
  return "changedTouches" in e ? [e.changedTouches[0].clientX, e.changedTouches[0].clientY] : [0, 0];
}, _r = function(e) {
  return [e.deltaX, e.deltaY];
}, Ir = function(e) {
  return e && "current" in e ? e.current : e;
}, zd = function(e, t) {
  return e[0] === t[0] && e[1] === t[1];
}, $d = function(e) {
  return `
  .block-interactivity-`.concat(e, ` {pointer-events: none;}
  .allow-interactivity-`).concat(e, ` {pointer-events: all;}
`);
}, Wd = 0, Xe = [];
function Bd(e) {
  var t = u.useRef([]), n = u.useRef([0, 0]), r = u.useRef(), o = u.useState(Wd++)[0], s = u.useState(ds)[0], i = u.useRef(e);
  u.useEffect(function() {
    i.current = e;
  }, [e]), u.useEffect(function() {
    if (e.inert) {
      document.body.classList.add("block-interactivity-".concat(o));
      var p = cd([e.lockRef.current], (e.shards || []).map(Ir), !0).filter(Boolean);
      return p.forEach(function(x) {
        return x.classList.add("allow-interactivity-".concat(o));
      }), function() {
        document.body.classList.remove("block-interactivity-".concat(o)), p.forEach(function(x) {
          return x.classList.remove("allow-interactivity-".concat(o));
        });
      };
    }
  }, [e.inert, e.lockRef.current, e.shards]);
  var a = u.useCallback(function(p, x) {
    if ("touches" in p && p.touches.length === 2 || p.type === "wheel" && p.ctrlKey)
      return !i.current.allowPinchZoom;
    var b = At(p), y = n.current, v = "deltaX" in p ? p.deltaX : y[0] - b[0], S = "deltaY" in p ? p.deltaY : y[1] - b[1], E, C = p.target, k = Math.abs(v) > Math.abs(S) ? "h" : "v";
    if ("touches" in p && k === "h" && C.type === "range")
      return !1;
    var N = window.getSelection(), L = N && N.anchorNode, F = L ? L === C || L.contains(C) : !1;
    if (F)
      return !1;
    var _ = Tr(k, C);
    if (!_)
      return !0;
    if (_ ? E = k : (E = k === "v" ? "h" : "v", _ = Tr(k, C)), !_)
      return !1;
    if (!r.current && "changedTouches" in p && (v || S) && (r.current = E), !E)
      return !0;
    var B = r.current || E;
    return Fd(B, x, p, B === "h" ? v : S);
  }, []), c = u.useCallback(function(p) {
    var x = p;
    if (!(!Xe.length || Xe[Xe.length - 1] !== s)) {
      var b = "deltaY" in x ? _r(x) : At(x), y = t.current.filter(function(E) {
        return E.name === x.type && (E.target === x.target || x.target === E.shadowParent) && zd(E.delta, b);
      })[0];
      if (y && y.should) {
        x.cancelable && x.preventDefault();
        return;
      }
      if (!y) {
        var v = (i.current.shards || []).map(Ir).filter(Boolean).filter(function(E) {
          return E.contains(x.target);
        }), S = v.length > 0 ? a(x, v[0]) : !i.current.noIsolation;
        S && x.cancelable && x.preventDefault();
      }
    }
  }, []), l = u.useCallback(function(p, x, b, y) {
    var v = { name: p, delta: x, target: b, should: y, shadowParent: Vd(b) };
    t.current.push(v), setTimeout(function() {
      t.current = t.current.filter(function(S) {
        return S !== v;
      });
    }, 1);
  }, []), f = u.useCallback(function(p) {
    n.current = At(p), r.current = void 0;
  }, []), m = u.useCallback(function(p) {
    l(p.type, _r(p), p.target, a(p, e.lockRef.current));
  }, []), h = u.useCallback(function(p) {
    l(p.type, At(p), p.target, a(p, e.lockRef.current));
  }, []);
  u.useEffect(function() {
    return Xe.push(s), e.setCallbacks({
      onScrollCapture: m,
      onWheelCapture: m,
      onTouchMoveCapture: h
    }), document.addEventListener("wheel", c, Ye), document.addEventListener("touchmove", c, Ye), document.addEventListener("touchstart", f, Ye), function() {
      Xe = Xe.filter(function(p) {
        return p !== s;
      }), document.removeEventListener("wheel", c, Ye), document.removeEventListener("touchmove", c, Ye), document.removeEventListener("touchstart", f, Ye);
    };
  }, []);
  var g = e.removeScrollBar, w = e.inert;
  return u.createElement(
    u.Fragment,
    null,
    w ? u.createElement(s, { styles: $d(o) }) : null,
    g ? u.createElement(Td, { noRelative: e.noRelative, gapMode: e.gapMode }) : null
  );
}
function Vd(e) {
  for (var t = null; e !== null; )
    e instanceof ShadowRoot && (t = e.host, e = e.host), e = e.parentNode;
  return t;
}
const Hd = vd(ls, Bd);
var ps = u.forwardRef(function(e, t) {
  return u.createElement(qt, ye({}, e, { ref: t, sideCar: Hd }));
});
ps.classNames = qt.classNames;
var Ud = [" ", "Enter", "ArrowUp", "ArrowDown"], jd = [" ", "Enter"], Ue = "Select", [Yt, Xt, Gd] = ro(Ue), [Ge] = nt(Ue, [
  Gd,
  Yo
]), Zt = Yo(), [Kd, ze] = Ge(Ue), [qd, Yd] = Ge(Ue), Xd = "SelectProvider";
function hs(e) {
  const {
    __scopeSelect: t,
    children: n,
    open: r,
    defaultOpen: o,
    onOpenChange: s,
    value: i,
    defaultValue: a,
    onValueChange: c,
    dir: l,
    name: f,
    autoComplete: m,
    disabled: h,
    required: g,
    form: w,
    // @ts-expect-error internal render prop used by `Select` to compose its default parts
    internal_do_not_use_render: p
  } = e, x = Zt(t), [b, y] = u.useState(null), [v, S] = u.useState(null), [E, C] = u.useState(!1), k = _n(l), [N, L] = ft({
    prop: r,
    defaultProp: o ?? !1,
    onChange: s,
    caller: Ue
  }), [F, _] = ft({
    prop: i,
    defaultProp: a,
    onChange: c,
    caller: Ue
  }), B = u.useRef(null), $ = u.useRef(F);
  u.useEffect(() => {
    const ee = w ? b == null ? void 0 : b.ownerDocument.getElementById(w) : b == null ? void 0 : b.form;
    if (ee instanceof HTMLFormElement) {
      const ce = () => _($.current);
      return ee.addEventListener("reset", ce), () => ee.removeEventListener("reset", ce);
    }
  }, [w, b, _]);
  const H = b ? !!w || !!b.closest("form") : !0, [U, O] = u.useState(/* @__PURE__ */ new Set()), G = bt(), M = Array.from(U).map((ee) => ee.props.value).join(";"), K = u.useCallback((ee) => {
    O((ce) => new Set(ce).add(ee));
  }, []), P = u.useCallback((ee) => {
    O((ce) => {
      const le = new Set(ce);
      return le.delete(ee), le;
    });
  }, []), me = {
    required: g,
    trigger: b,
    onTriggerChange: y,
    valueNode: v,
    onValueNodeChange: S,
    valueNodeHasChildren: E,
    onValueNodeHasChildrenChange: C,
    contentId: G,
    value: F,
    onValueChange: _,
    open: N,
    onOpenChange: L,
    dir: k,
    triggerPointerDownPosRef: B,
    disabled: h,
    name: f,
    autoComplete: m,
    form: w,
    nativeOptions: U,
    nativeSelectKey: M,
    isFormControl: H
  };
  return /* @__PURE__ */ d(Zl, { ...x, children: /* @__PURE__ */ d(Kd, { scope: t, ...me, children: /* @__PURE__ */ d(Yt.Provider, { scope: t, children: /* @__PURE__ */ d(
    qd,
    {
      scope: t,
      onNativeOptionAdd: K,
      onNativeOptionRemove: P,
      children: hu(p) ? p(me) : n
    }
  ) }) }) });
}
hs.displayName = Xd;
var gs = (e) => {
  const { __scopeSelect: t, children: n, ...r } = e;
  return /* @__PURE__ */ d(
    hs,
    {
      __scopeSelect: t,
      ...r,
      internal_do_not_use_render: ({ isFormControl: o }) => /* @__PURE__ */ R(Bt, { children: [
        n,
        o ? /* @__PURE__ */ d(
          Fs,
          {
            __scopeSelect: t
          }
        ) : null
      ] })
    }
  );
};
gs.displayName = Ue;
var vs = "SelectTrigger", Un = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, disabled: r = !1, ...o } = e, s = Zt(n), i = ze(vs, n), a = i.disabled || r, c = re(t, i.onTriggerChange), l = Xt(n), f = u.useRef("touch"), [m, h, g] = zs((p) => {
      const x = l().filter((v) => !v.disabled), b = x.find((v) => v.value === i.value), y = $s(x, p, b);
      y !== void 0 && i.onValueChange(y.value);
    }), w = (p) => {
      a || (i.onOpenChange(!0), g()), p && (i.triggerPointerDownPosRef.current = {
        x: Math.round(p.pageX),
        y: Math.round(p.pageY)
      });
    };
    return /* @__PURE__ */ d(Ql, { asChild: !0, ...s, children: /* @__PURE__ */ d(
      q.button,
      {
        type: "button",
        role: "combobox",
        "aria-controls": i.open ? i.contentId : void 0,
        "aria-expanded": i.open,
        "aria-required": i.required,
        "aria-autocomplete": "none",
        dir: i.dir,
        "data-state": i.open ? "open" : "closed",
        disabled: a,
        "data-disabled": a ? "" : void 0,
        "data-placeholder": Qt(i.value) ? "" : void 0,
        ...o,
        ref: c,
        onClick: Y(o.onClick, (p) => {
          p.currentTarget.focus(), f.current !== "mouse" && w(p);
        }),
        onPointerDown: Y(o.onPointerDown, (p) => {
          f.current = p.pointerType;
          const x = p.target;
          x.hasPointerCapture(p.pointerId) && x.releasePointerCapture(p.pointerId), p.button === 0 && p.ctrlKey === !1 && p.pointerType === "mouse" && (w(p), p.preventDefault());
        }),
        onKeyDown: Y(o.onKeyDown, (p) => {
          const x = m.current !== "";
          !(p.ctrlKey || p.altKey || p.metaKey) && p.key.length === 1 && h(p.key), !(x && p.key === " ") && Ud.includes(p.key) && (w(), p.preventDefault());
        })
      }
    ) });
  }
);
Un.displayName = vs;
var bs = "SelectValue", ys = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, className: r, style: o, children: s, placeholder: i = "", ...a } = e, c = ze(bs, n), { onValueNodeHasChildrenChange: l } = c, f = s !== void 0, m = re(t, c.onValueNodeChange);
    se(() => {
      l(f);
    }, [l, f]);
    const h = Qt(c.value);
    return /* @__PURE__ */ d(
      q.span,
      {
        ...a,
        asChild: h ? !1 : a.asChild,
        ref: m,
        style: { pointerEvents: "none" },
        children: /* @__PURE__ */ d(u.Fragment, { children: h ? i : s }, h ? "placeholder" : "value")
      }
    );
  }
);
ys.displayName = bs;
var Zd = "SelectIcon", ws = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, children: r, ...o } = e;
    return /* @__PURE__ */ d(q.span, { "aria-hidden": !0, ...o, ref: t, children: r || "▼" });
  }
);
ws.displayName = Zd;
var xs = "SelectPortal", [Qd, Jd] = Ge(xs, {
  forceMount: void 0
}), Cs = (e) => {
  const { __scopeSelect: t, forceMount: n, ...r } = e;
  return /* @__PURE__ */ d(Qd, { scope: e.__scopeSelect, forceMount: n, children: /* @__PURE__ */ d(rs, { asChild: !0, ...r }) });
};
Cs.displayName = xs;
var Le = "SelectContent", jn = u.forwardRef(
  (e, t) => {
    const n = Jd(Le, e.__scopeSelect), { forceMount: r = n.forceMount, ...o } = e, s = ze(Le, e.__scopeSelect), [i, a] = u.useState();
    return se(() => {
      a(new DocumentFragment());
    }, []), /* @__PURE__ */ d(In, { present: r || s.open, children: ({ present: c }) => c ? /* @__PURE__ */ d(Es, { ...o, ref: t }) : /* @__PURE__ */ d(Ss, { ...o, fragment: i }) });
  }
);
jn.displayName = Le;
var Ss = u.forwardRef((e, t) => {
  const { __scopeSelect: n, children: r, fragment: o } = e;
  return o ? vt.createPortal(
    /* @__PURE__ */ d(Ns, { scope: n, children: /* @__PURE__ */ d(Yt.Slot, { scope: n, children: /* @__PURE__ */ d("div", { ref: t, children: r }) }) }),
    o
  ) : null;
});
Ss.displayName = "SelectContentFragment";
var ge = 10, [Ns, $e] = Ge(Le), eu = "SelectContentImpl", tu = /* @__PURE__ */ ut("SelectContent.RemoveScroll"), Es = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n } = e, {
      position: r = "item-aligned",
      onCloseAutoFocus: o,
      onEscapeKeyDown: s,
      onPointerDownOutside: i,
      //
      // PopperContent props
      side: a,
      sideOffset: c,
      align: l,
      alignOffset: f,
      arrowPadding: m,
      collisionBoundary: h,
      collisionPadding: g,
      sticky: w,
      hideWhenDetached: p,
      avoidCollisions: x,
      //
      ...b
    } = e, y = ze(Le, n), [v, S] = u.useState(null), [E, C] = u.useState(null), k = re(t, S), [N, L] = u.useState(null), [F, _] = u.useState(
      null
    ), B = Xt(n), [$, H] = u.useState(!1), U = u.useRef(!1);
    u.useEffect(() => {
      if (v) return ad(v);
    }, [v]), Tc();
    const O = u.useCallback(
      (D) => {
        const [Z, ...V] = B().map((j) => j.ref.current), [X] = V.slice(-1), W = document.activeElement;
        for (const j of D)
          if (j === W || (j == null || j.scrollIntoView({ block: "nearest" }), j === Z && E && (E.scrollTop = 0), j === X && E && (E.scrollTop = E.scrollHeight), j == null || j.focus(), document.activeElement !== W)) return;
      },
      [B, E]
    ), G = u.useCallback(
      () => O([N, v]),
      [O, N, v]
    );
    u.useEffect(() => {
      $ && G();
    }, [$, G]);
    const { onOpenChange: M, triggerPointerDownPosRef: K } = y;
    u.useEffect(() => {
      if (v) {
        let D = { x: 0, y: 0 };
        const Z = (X) => {
          var W, j;
          D = {
            x: Math.abs(Math.round(X.pageX) - (((W = K.current) == null ? void 0 : W.x) ?? 0)),
            y: Math.abs(Math.round(X.pageY) - (((j = K.current) == null ? void 0 : j.y) ?? 0))
          };
        }, V = (X) => {
          D.x <= 10 && D.y <= 10 ? X.preventDefault() : X.composedPath().includes(v) || M(!1), document.removeEventListener("pointermove", Z), K.current = null;
        };
        return K.current !== null && (document.addEventListener("pointermove", Z), document.addEventListener("pointerup", V, { capture: !0, once: !0 })), () => {
          document.removeEventListener("pointermove", Z), document.removeEventListener("pointerup", V, { capture: !0 });
        };
      }
    }, [v, M, K]), u.useEffect(() => {
      const D = () => M(!1);
      return window.addEventListener("blur", D), window.addEventListener("resize", D), () => {
        window.removeEventListener("blur", D), window.removeEventListener("resize", D);
      };
    }, [M]);
    const [P, me] = zs((D) => {
      const Z = B().filter((W) => !W.disabled), V = Z.find((W) => W.ref.current === document.activeElement), X = $s(Z, D, V);
      X && setTimeout(() => {
        var W;
        return (W = X.ref.current) == null ? void 0 : W.focus();
      });
    }), ee = u.useCallback(
      (D, Z, V) => {
        const X = !U.current && !V;
        (y.value !== void 0 && y.value === Z || X) && (L(D), X && (U.current = !0));
      },
      [y.value]
    ), ce = u.useCallback(() => v == null ? void 0 : v.focus(), [v]), le = u.useCallback(
      (D, Z, V) => {
        const X = !U.current && !V;
        (y.value !== void 0 && y.value === Z || X) && _(D);
      },
      [y.value]
    ), J = r === "popper" ? Cn : ks, Q = J === Cn ? {
      side: a,
      sideOffset: c,
      align: l,
      alignOffset: f,
      arrowPadding: m,
      collisionBoundary: h,
      collisionPadding: g,
      sticky: w,
      hideWhenDetached: p,
      avoidCollisions: x
    } : {};
    return /* @__PURE__ */ d(
      Ns,
      {
        scope: n,
        content: v,
        viewport: E,
        onViewportChange: C,
        itemRefCallback: ee,
        selectedItem: N,
        onItemLeave: ce,
        itemTextRefCallback: le,
        focusSelectedItem: G,
        selectedItemText: F,
        position: r,
        isPositioned: $,
        searchRef: P,
        children: /* @__PURE__ */ d(ps, { as: tu, allowPinchZoom: !0, children: /* @__PURE__ */ d(
          Oo,
          {
            asChild: !0,
            trapped: y.open,
            onMountAutoFocus: (D) => {
              D.preventDefault();
            },
            onUnmountAutoFocus: Y(o, (D) => {
              var Z;
              (Z = y.trigger) == null || Z.focus({ preventScroll: !0 }), D.preventDefault();
            }),
            children: /* @__PURE__ */ d(
              _o,
              {
                asChild: !0,
                disableOutsidePointerEvents: !0,
                onEscapeKeyDown: s,
                onPointerDownOutside: i,
                onFocusOutside: (D) => D.preventDefault(),
                onDismiss: () => y.onOpenChange(!1),
                children: /* @__PURE__ */ d(
                  J,
                  {
                    role: "listbox",
                    id: y.contentId,
                    "data-state": y.open ? "open" : "closed",
                    dir: y.dir,
                    onContextMenu: (D) => D.preventDefault(),
                    ...b,
                    ...Q,
                    onPlaced: () => H(!0),
                    ref: k,
                    style: {
                      // flex layout so we can place the scroll buttons properly
                      display: "flex",
                      flexDirection: "column",
                      // reset the outline by default as the content MAY get focused
                      outline: "none",
                      ...b.style
                    },
                    onKeyDown: Y(b.onKeyDown, (D) => {
                      const Z = D.ctrlKey || D.altKey || D.metaKey;
                      if (D.key === "Tab" && D.preventDefault(), !Z && D.key.length === 1 && me(D.key), ["ArrowUp", "ArrowDown", "Home", "End"].includes(D.key)) {
                        let X = B().filter((W) => !W.disabled).map((W) => W.ref.current);
                        if (["ArrowUp", "End"].includes(D.key) && (X = X.slice().reverse()), ["ArrowUp", "ArrowDown"].includes(D.key)) {
                          const W = D.target, j = X.indexOf(W);
                          X = X.slice(j + 1);
                        }
                        setTimeout(() => O(X)), D.preventDefault();
                      }
                    })
                  }
                )
              }
            )
          }
        ) })
      }
    );
  }
);
Es.displayName = eu;
var nu = "SelectItemAlignedPosition", ks = u.forwardRef((e, t) => {
  const { __scopeSelect: n, onPlaced: r, ...o } = e, s = ze(Le, n), i = $e(Le, n), [a, c] = u.useState(null), [l, f] = u.useState(null), m = re(t, f), h = Xt(n), g = u.useRef(!1), w = u.useRef(!0), { viewport: p, selectedItem: x, selectedItemText: b, focusSelectedItem: y } = i, v = u.useCallback(() => {
    if (s.trigger && s.valueNode && a && l && p && x && b) {
      const k = s.trigger.getBoundingClientRect(), N = l.getBoundingClientRect(), L = s.valueNode.getBoundingClientRect(), F = b.getBoundingClientRect();
      if (s.dir !== "rtl") {
        const W = F.left - N.left, j = L.left - W, de = k.left - j, ae = k.width + de, Ke = Math.max(ae, N.width), st = window.innerWidth - ge, it = dr(j, [
          ge,
          // Prevents the content from going off the starting edge of the
          // viewport. It may still go off the ending edge, but this can be
          // controlled by the user since they may want to manage overflow in a
          // specific way.
          // https://github.com/radix-ui/primitives/issues/2049
          Math.max(ge, st - Ke)
        ]);
        a.style.minWidth = ae + "px", a.style.left = it + "px";
      } else {
        const W = N.right - F.right, j = window.innerWidth - L.right - W, de = window.innerWidth - k.right - j, ae = k.width + de, Ke = Math.max(ae, N.width), st = window.innerWidth - ge, it = dr(j, [
          ge,
          Math.max(ge, st - Ke)
        ]);
        a.style.minWidth = ae + "px", a.style.right = it + "px";
      }
      const _ = h(), B = window.innerHeight - ge * 2, $ = p.scrollHeight, H = window.getComputedStyle(l), U = parseInt(H.borderTopWidth, 10), O = parseInt(H.paddingTop, 10), G = parseInt(H.borderBottomWidth, 10), M = parseInt(H.paddingBottom, 10), K = U + O + $ + M + G, P = Math.min(x.offsetHeight * 5, K), me = window.getComputedStyle(p), ee = parseInt(me.paddingTop, 10), ce = parseInt(me.paddingBottom, 10), le = k.top + k.height / 2 - ge, J = B - le, Q = x.offsetHeight / 2, D = x.offsetTop + Q, Z = U + O + D, V = K - Z;
      if (Z <= le) {
        const W = _.length > 0 && x === _[_.length - 1].ref.current;
        a.style.bottom = "0px";
        const j = l.clientHeight - p.offsetTop - p.offsetHeight, de = Math.max(
          J,
          Q + // viewport might have padding bottom, include it to avoid a scrollable viewport
          (W ? ce : 0) + j + G
        ), ae = Z + de;
        a.style.height = ae + "px";
      } else {
        const W = _.length > 0 && x === _[0].ref.current;
        a.style.top = "0px";
        const de = Math.max(
          le,
          U + p.offsetTop + // viewport might have padding top, include it to avoid a scrollable viewport
          (W ? ee : 0) + Q
        ) + V;
        a.style.height = de + "px", p.scrollTop = Z - le + p.offsetTop;
      }
      a.style.margin = `${ge}px 0`, a.style.minHeight = P + "px", a.style.maxHeight = B + "px", r == null || r(), requestAnimationFrame(() => g.current = !0);
    }
  }, [
    h,
    s.trigger,
    s.valueNode,
    a,
    l,
    p,
    x,
    b,
    s.dir,
    r
  ]);
  se(() => v(), [v]);
  const [S, E] = u.useState();
  se(() => {
    l && E(window.getComputedStyle(l).zIndex);
  }, [l]);
  const C = u.useCallback(
    (k) => {
      k && w.current === !0 && (v(), y == null || y(), w.current = !1);
    },
    [v, y]
  );
  return /* @__PURE__ */ d(
    ou,
    {
      scope: n,
      contentWrapper: a,
      shouldExpandOnScrollRef: g,
      onScrollButtonChange: C,
      children: /* @__PURE__ */ d(
        "div",
        {
          ref: c,
          style: {
            display: "flex",
            flexDirection: "column",
            position: "fixed",
            zIndex: S
          },
          children: /* @__PURE__ */ d(
            q.div,
            {
              ...o,
              ref: m,
              style: {
                // When we get the height of the content, it includes borders. If we were to set
                // the height without having `boxSizing: 'border-box'` it would be too big.
                boxSizing: "border-box",
                // We need to ensure the content doesn't get taller than the wrapper
                maxHeight: "100%",
                ...o.style
              }
            }
          )
        }
      )
    }
  );
});
ks.displayName = nu;
var ru = "SelectPopperPosition", Cn = u.forwardRef((e, t) => {
  const {
    __scopeSelect: n,
    align: r = "start",
    collisionPadding: o = ge,
    ...s
  } = e, i = Zt(n);
  return /* @__PURE__ */ d(
    Jl,
    {
      ...i,
      ...s,
      ref: t,
      align: r,
      collisionPadding: o,
      style: {
        // Ensure border-box for floating-ui calculations
        boxSizing: "border-box",
        ...s.style,
        "--radix-select-content-transform-origin": "var(--radix-popper-transform-origin)",
        "--radix-select-content-available-width": "var(--radix-popper-available-width)",
        "--radix-select-content-available-height": "var(--radix-popper-available-height)",
        "--radix-select-trigger-width": "var(--radix-popper-anchor-width)",
        "--radix-select-trigger-height": "var(--radix-popper-anchor-height)"
      }
    }
  );
});
Cn.displayName = ru;
var [ou, Gn] = Ge(Le, {}), Sn = "SelectViewport", Rs = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, nonce: r, ...o } = e, s = $e(Sn, n), i = Gn(Sn, n), a = re(t, s.onViewportChange), c = u.useRef(0);
    return /* @__PURE__ */ R(Bt, { children: [
      /* @__PURE__ */ d(
        "style",
        {
          dangerouslySetInnerHTML: {
            __html: "[data-radix-select-viewport]{scrollbar-width:none;-ms-overflow-style:none;-webkit-overflow-scrolling:touch;}[data-radix-select-viewport]::-webkit-scrollbar{display:none}"
          },
          nonce: r
        }
      ),
      /* @__PURE__ */ d(Yt.Slot, { scope: n, children: /* @__PURE__ */ d(
        q.div,
        {
          "data-radix-select-viewport": "",
          role: "presentation",
          ...o,
          ref: a,
          style: {
            // we use position: 'relative' here on the `viewport` so that when we call
            // `selectedItem.offsetTop` in calculations, the offset is relative to the viewport
            // (independent of the scrollUpButton).
            position: "relative",
            flex: 1,
            // Viewport should only be scrollable in the vertical direction.
            // This won't work in vertical writing modes, so we'll need to
            // revisit this if/when that is supported
            // https://developer.chrome.com/blog/vertical-form-controls
            overflow: "hidden auto",
            ...o.style
          },
          onScroll: Y(o.onScroll, (l) => {
            const f = l.currentTarget, { contentWrapper: m, shouldExpandOnScrollRef: h } = i;
            if (h != null && h.current && m) {
              const g = Math.abs(c.current - f.scrollTop);
              if (g > 0) {
                const w = window.innerHeight - ge * 2, p = parseFloat(m.style.minHeight), x = parseFloat(m.style.height), b = Math.max(p, x);
                if (b < w) {
                  const y = b + g, v = Math.min(w, y), S = y - v;
                  m.style.height = v + "px", m.style.bottom === "0px" && (f.scrollTop = S > 0 ? S : 0, m.style.justifyContent = "flex-end");
                }
              }
            }
            c.current = f.scrollTop;
          })
        }
      ) })
    ] });
  }
);
Rs.displayName = Sn;
var Ps = "SelectGroup", [su, iu] = Ge(Ps), au = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, ...r } = e, o = bt();
    return /* @__PURE__ */ d(su, { scope: n, id: o, children: /* @__PURE__ */ d(q.div, { role: "group", "aria-labelledby": o, ...r, ref: t }) });
  }
);
au.displayName = Ps;
var As = "SelectLabel", cu = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, ...r } = e, o = iu(As, n);
    return /* @__PURE__ */ d(q.div, { id: o.id, ...r, ref: t });
  }
);
cu.displayName = As;
var zt = "SelectItem", [lu, Ts] = Ge(zt), Kn = u.forwardRef(
  (e, t) => {
    const {
      __scopeSelect: n,
      value: r,
      disabled: o = !1,
      textValue: s,
      ...i
    } = e, a = ze(zt, n), c = $e(zt, n), l = a.value === r, [f, m] = u.useState(s ?? ""), [h, g] = u.useState(!1), w = xe(
      (v) => {
        var S;
        return (S = c.itemRefCallback) == null ? void 0 : S.call(c, v, r, o);
      }
    ), p = re(t, w), x = bt(), b = u.useRef("touch"), y = () => {
      o || (a.onValueChange(r), a.onOpenChange(!1));
    };
    return /* @__PURE__ */ d(
      lu,
      {
        scope: n,
        value: r,
        disabled: o,
        textId: x,
        isSelected: l,
        onItemTextChange: u.useCallback((v) => {
          m((S) => S || ((v == null ? void 0 : v.textContent) ?? "").trim());
        }, []),
        children: /* @__PURE__ */ d(
          Yt.ItemSlot,
          {
            scope: n,
            value: r,
            disabled: o,
            textValue: f,
            children: /* @__PURE__ */ d(
              q.div,
              {
                role: "option",
                "aria-labelledby": x,
                "data-highlighted": h ? "" : void 0,
                "aria-selected": l && h,
                "data-state": l ? "checked" : "unchecked",
                "aria-disabled": o || void 0,
                "data-disabled": o ? "" : void 0,
                tabIndex: o ? void 0 : -1,
                ...i,
                ref: p,
                onFocus: Y(i.onFocus, () => g(!0)),
                onBlur: Y(i.onBlur, () => g(!1)),
                onClick: Y(i.onClick, () => {
                  b.current !== "mouse" && y();
                }),
                onPointerUp: Y(i.onPointerUp, () => {
                  b.current === "mouse" && y();
                }),
                onPointerDown: Y(i.onPointerDown, (v) => {
                  b.current = v.pointerType;
                }),
                onPointerMove: Y(i.onPointerMove, (v) => {
                  var S;
                  b.current = v.pointerType, o ? (S = c.onItemLeave) == null || S.call(c) : b.current === "mouse" && v.currentTarget.focus({ preventScroll: !0 });
                }),
                onPointerLeave: Y(i.onPointerLeave, (v) => {
                  var S;
                  v.currentTarget === document.activeElement && ((S = c.onItemLeave) == null || S.call(c));
                }),
                onKeyDown: Y(i.onKeyDown, (v) => {
                  var E;
                  o || v.target !== v.currentTarget || ((E = c.searchRef) == null ? void 0 : E.current) !== "" && v.key === " " || (jd.includes(v.key) && y(), v.key === " " && v.preventDefault());
                })
              }
            )
          }
        )
      }
    );
  }
);
Kn.displayName = zt;
var dt = "SelectItemText", _s = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, className: r, style: o, ...s } = e, i = ze(dt, n), a = $e(dt, n), c = Ts(dt, n), l = Yd(dt, n), [f, m] = u.useState(null), h = xe(
      (y) => {
        var v;
        return (v = a.itemTextRefCallback) == null ? void 0 : v.call(a, y, c.value, c.disabled);
      }
    ), g = re(
      t,
      m,
      c.onItemTextChange,
      h
    ), w = f == null ? void 0 : f.textContent, p = u.useMemo(
      () => /* @__PURE__ */ d("option", { value: c.value, disabled: c.disabled, children: w }, c.value),
      [c.disabled, c.value, w]
    ), { onNativeOptionAdd: x, onNativeOptionRemove: b } = l;
    return se(() => (x(p), () => b(p)), [x, b, p]), /* @__PURE__ */ R(Bt, { children: [
      /* @__PURE__ */ d(q.span, { id: c.textId, ...s, ref: g }),
      c.isSelected && i.valueNode && !i.valueNodeHasChildren && !Qt(i.value) ? vt.createPortal(s.children, i.valueNode) : null
    ] });
  }
);
_s.displayName = dt;
var Is = "SelectItemIndicator", Os = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, ...r } = e;
    return Ts(Is, n).isSelected ? /* @__PURE__ */ d(q.span, { "aria-hidden": !0, ...r, ref: t }) : null;
  }
);
Os.displayName = Is;
var Nn = "SelectScrollUpButton", du = u.forwardRef((e, t) => {
  const n = $e(Nn, e.__scopeSelect), r = Gn(Nn, e.__scopeSelect), [o, s] = u.useState(!1), i = re(t, r.onScrollButtonChange);
  return se(() => {
    if (n.viewport && n.isPositioned) {
      let a = function() {
        const l = c.scrollTop > 0;
        s(l);
      };
      const c = n.viewport;
      return a(), c.addEventListener("scroll", a), () => c.removeEventListener("scroll", a);
    }
  }, [n.viewport, n.isPositioned]), o ? /* @__PURE__ */ d(
    Ms,
    {
      ...e,
      ref: i,
      onAutoScroll: () => {
        const { viewport: a, selectedItem: c } = n;
        a && c && (a.scrollTop = a.scrollTop - c.offsetHeight);
      }
    }
  ) : null;
});
du.displayName = Nn;
var En = "SelectScrollDownButton", uu = u.forwardRef((e, t) => {
  const n = $e(En, e.__scopeSelect), r = Gn(En, e.__scopeSelect), [o, s] = u.useState(!1), i = re(t, r.onScrollButtonChange);
  return se(() => {
    if (n.viewport && n.isPositioned) {
      let a = function() {
        const l = c.scrollHeight - c.clientHeight, f = Math.ceil(c.scrollTop) < l;
        s(f);
      };
      const c = n.viewport;
      return a(), c.addEventListener("scroll", a), () => c.removeEventListener("scroll", a);
    }
  }, [n.viewport, n.isPositioned]), o ? /* @__PURE__ */ d(
    Ms,
    {
      ...e,
      ref: i,
      onAutoScroll: () => {
        const { viewport: a, selectedItem: c } = n;
        a && c && (a.scrollTop = a.scrollTop + c.offsetHeight);
      }
    }
  ) : null;
});
uu.displayName = En;
var Ms = u.forwardRef((e, t) => {
  const { __scopeSelect: n, onAutoScroll: r, ...o } = e, s = $e("SelectScrollButton", n), i = u.useRef(null), a = Xt(n), c = u.useCallback(() => {
    i.current !== null && (window.clearInterval(i.current), i.current = null);
  }, []);
  return u.useEffect(() => () => c(), [c]), se(() => {
    var f;
    const l = a().find((m) => m.ref.current === document.activeElement);
    (f = l == null ? void 0 : l.ref.current) == null || f.scrollIntoView({ block: "nearest" });
  }, [a]), /* @__PURE__ */ d(
    q.div,
    {
      "aria-hidden": !0,
      ...o,
      ref: t,
      style: { flexShrink: 0, ...o.style },
      onPointerDown: Y(o.onPointerDown, () => {
        i.current === null && (i.current = window.setInterval(r, 50));
      }),
      onPointerMove: Y(o.onPointerMove, () => {
        var l;
        (l = s.onItemLeave) == null || l.call(s), i.current === null && (i.current = window.setInterval(r, 50));
      }),
      onPointerLeave: Y(o.onPointerLeave, () => {
        c();
      })
    }
  );
}), fu = "SelectSeparator", mu = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, ...r } = e;
    return /* @__PURE__ */ d(q.div, { "aria-hidden": !0, ...r, ref: t });
  }
);
mu.displayName = fu;
var Ls = "SelectArrow", pu = u.forwardRef(
  (e, t) => {
    const { __scopeSelect: n, ...r } = e, o = Zt(n);
    return $e(Ls, n).position === "popper" ? /* @__PURE__ */ d(ed, { ...o, ...r, ref: t }) : null;
  }
);
pu.displayName = Ls;
var Ds = "SelectBubbleInput", Fs = u.forwardRef(
  ({ __scopeSelect: e, ...t }, n) => {
    const r = ze(Ds, e), { value: o, onValueChange: s, required: i, disabled: a, name: c, autoComplete: l, form: f } = r, { nativeOptions: m, nativeSelectKey: h } = r, g = u.useRef(null), w = re(n, g), p = o ?? "", x = os(p), b = Array.from(m).some(
      (y) => (y.props.value ?? "") === ""
    );
    return u.useEffect(() => {
      const y = g.current;
      if (!y) return;
      const v = window.HTMLSelectElement.prototype, E = Object.getOwnPropertyDescriptor(
        v,
        "value"
      ).set;
      if (x !== p && E) {
        const C = new Event("change", { bubbles: !0 });
        E.call(y, p), y.dispatchEvent(C);
      }
    }, [x, p]), /* @__PURE__ */ R(
      q.select,
      {
        "aria-hidden": !0,
        required: i,
        tabIndex: -1,
        name: c,
        autoComplete: l,
        disabled: a,
        form: f,
        onChange: (y) => s(y.target.value),
        ...t,
        style: { ...ss, ...t.style },
        ref: w,
        defaultValue: p,
        children: [
          Qt(o) && !b ? /* @__PURE__ */ d("option", { value: "" }) : null,
          Array.from(m)
        ]
      },
      h
    );
  }
);
Fs.displayName = Ds;
function hu(e) {
  return typeof e == "function";
}
function Qt(e) {
  return e === "" || e === void 0;
}
function zs(e) {
  const t = xe(e), n = u.useRef(""), r = u.useRef(0), o = u.useCallback(
    (i) => {
      const a = n.current + i;
      t(a), function c(l) {
        n.current = l, window.clearTimeout(r.current), l !== "" && (r.current = window.setTimeout(() => c(""), 1e3));
      }(a);
    },
    [t]
  ), s = u.useCallback(() => {
    n.current = "", window.clearTimeout(r.current);
  }, []);
  return u.useEffect(() => () => window.clearTimeout(r.current), []), [n, o, s];
}
function $s(e, t, n) {
  const o = t.length > 1 && Array.from(t).every((l) => l === t[0]) ? t[0] : t, s = n ? e.indexOf(n) : -1;
  let i = gu(e, Math.max(s, 0));
  o.length === 1 && (i = i.filter((l) => l !== n));
  const c = i.find(
    (l) => l.textValue.toLowerCase().startsWith(o.toLowerCase())
  );
  return c !== n ? c : void 0;
}
function gu(e, t) {
  return e.map((n, r) => e[(t + r) % e.length]);
}
const $t = gs, Wt = ys, ht = u.forwardRef(({ className: e, children: t, ...n }, r) => /* @__PURE__ */ R(
  Un,
  {
    ref: r,
    className: ne(
      "flex h-11 w-full items-center justify-between rounded-xl border border-input bg-card px-4 py-2 text-base text-foreground shadow-sm",
      "focus:outline-none focus:border-ring focus:ring-2 focus:ring-ring/40",
      "disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1",
      e
    ),
    ...n,
    children: [
      t,
      /* @__PURE__ */ d(ws, { asChild: !0, children: /* @__PURE__ */ d(ki, { className: "size-4 opacity-60" }) })
    ]
  }
));
ht.displayName = Un.displayName;
const gt = u.forwardRef(({ className: e, children: t, position: n = "popper", ...r }, o) => /* @__PURE__ */ d(Cs, { children: /* @__PURE__ */ d(
  jn,
  {
    ref: o,
    position: n,
    className: ne(
      "relative z-50 max-h-96 min-w-[8rem] overflow-hidden rounded-xl border border-border bg-popover text-popover-foreground shadow-lg",
      "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
      n === "popper" && "data-[side=bottom]:translate-y-1 data-[side=top]:-translate-y-1",
      e
    ),
    ...r,
    children: /* @__PURE__ */ d(
      Rs,
      {
        className: ne(
          "p-1",
          n === "popper" && "h-[var(--radix-select-trigger-height)] w-full min-w-[var(--radix-select-trigger-width)]"
        ),
        children: t
      }
    )
  }
) }));
gt.displayName = jn.displayName;
const tt = u.forwardRef(({ className: e, children: t, ...n }, r) => /* @__PURE__ */ R(
  Kn,
  {
    ref: r,
    className: ne(
      "relative flex w-full cursor-pointer select-none items-center rounded-lg py-2 pl-3 pr-8 text-base outline-none",
      "focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      e
    ),
    ...n,
    children: [
      /* @__PURE__ */ d("span", { className: "absolute right-2 flex size-4 items-center justify-center", children: /* @__PURE__ */ d(Os, { children: /* @__PURE__ */ d(Ci, { className: "size-4" }) }) }),
      /* @__PURE__ */ d(_s, { children: t })
    ]
  }
));
tt.displayName = Kn.displayName;
function vu({ embedded: e } = {}) {
  const t = fe(), n = Ae(), [r, o] = I(null), [s, i] = I(null), [a, c] = I(null), [l, f] = I(!1), [m, h] = I(null), [g, w] = I(null), [p, x] = I(null);
  async function b() {
    try {
      const [y, v] = await Promise.all([
        n.list("center"),
        n.list("room")
      ]);
      o(y), i(v);
    } catch (y) {
      c(y.message);
    }
  }
  return Je(() => {
    b();
  }, []), l || m ? /* @__PURE__ */ d(bu, { initial: m, onDone: () => {
    f(!1), h(null), b();
  } }) : g !== null || p ? /* @__PURE__ */ d(yu, { centerId: g ?? (p == null ? void 0 : p.center_id) ?? "", initial: p, centers: r ?? [], onDone: () => {
    w(null), x(null), b();
  } }) : /* @__PURE__ */ R("main", { className: e ? "" : "pb-24", children: [
    !e && /* @__PURE__ */ d(he, { children: t("admin.schools") }),
    a && /* @__PURE__ */ d("p", { role: "alert", className: "px-4 py-2 text-sm text-destructive", children: t("common.error_generic") }),
    /* @__PURE__ */ R("section", { className: "px-4 pt-1", children: [
      /* @__PURE__ */ R("header", { className: "flex items-center justify-between pb-2", children: [
        /* @__PURE__ */ d("h2", { className: "text-base font-semibold tracking-tight text-foreground", children: t("center.list.title") }),
        /* @__PURE__ */ R(ie, { size: "pill", onClick: () => f(!0), children: [
          /* @__PURE__ */ d(et, {}),
          " ",
          t("common.add")
        ] })
      ] }),
      r != null && r.length ? /* @__PURE__ */ d("ul", { className: "divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card shadow-sm", children: r.map((y) => /* @__PURE__ */ R("li", { className: "flex items-center justify-between gap-3 px-4 py-3", children: [
        /* @__PURE__ */ R("span", { className: "min-w-0", children: [
          /* @__PURE__ */ d("span", { className: "block truncate font-medium text-foreground", children: y.name }),
          y.address && /* @__PURE__ */ d("span", { className: "block truncate text-[13px] text-muted-foreground", children: y.address })
        ] }),
        /* @__PURE__ */ d("span", { className: "text-xs uppercase tracking-wide text-muted-foreground", children: y.default_locale ?? "" })
      ] }, y.id)) }) : /* @__PURE__ */ d("p", { className: "py-4 text-[15px] text-muted-foreground", children: t("center.empty") })
    ] }),
    /* @__PURE__ */ R("section", { className: "mt-8 px-4", children: [
      /* @__PURE__ */ d("h2", { className: "pb-2 text-base font-semibold tracking-tight text-foreground", children: t("room.list.title") }),
      s != null && s.length ? /* @__PURE__ */ d("ul", { className: "divide-y divide-border overflow-hidden rounded-2xl border border-border bg-card shadow-sm", children: s.map((y) => {
        const v = r == null ? void 0 : r.find((S) => S.id === y.center_id);
        return /* @__PURE__ */ d("li", { className: "flex items-center justify-between gap-3 px-4 py-3", children: /* @__PURE__ */ R("span", { className: "min-w-0", children: [
          /* @__PURE__ */ d("span", { className: "block truncate font-medium text-foreground", children: y.name }),
          /* @__PURE__ */ d("span", { className: "block truncate text-[13px] text-muted-foreground", children: (v == null ? void 0 : v.name) ?? y.center_id })
        ] }) }, y.id);
      }) }) : /* @__PURE__ */ d("p", { className: "py-4 text-[15px] text-muted-foreground", children: t("room.empty") }),
      r == null ? void 0 : r.map((y) => /* @__PURE__ */ R(
        ie,
        {
          variant: "outline",
          onClick: () => w(y.id),
          className: "mt-2 w-full justify-start border-dashed text-primary",
          children: [
            /* @__PURE__ */ d(et, {}),
            " ",
            t("common.add"),
            " ",
            t("room.name").toLowerCase(),
            " → ",
            y.name
          ]
        },
        y.id
      ))
    ] })
  ] });
}
function bu({ initial: e, onDone: t }) {
  const n = fe(), r = Ae(), [o, s] = I((e == null ? void 0 : e.name) ?? ""), [i, a] = I((e == null ? void 0 : e.address) ?? ""), [c, l] = I((e == null ? void 0 : e.phone) ?? ""), [f, m] = I((e == null ? void 0 : e.email) ?? ""), [h, g] = I((e == null ? void 0 : e.default_locale) ?? "en"), [w, p] = I(!1), [x, b] = I(null);
  async function y() {
    p(!0), b(null);
    try {
      const v = (e == null ? void 0 : e.id) ?? (o.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "center");
      await r.run("center.create", { id: v, name: o, address: i, phone: c, email: f, default_locale: h }), t();
    } catch {
      b(n("common.error_generic"));
    } finally {
      p(!1);
    }
  }
  return /* @__PURE__ */ R("main", { className: "pb-24", children: [
    /* @__PURE__ */ d(he, { children: n(e ? "center.editor.title.edit" : "center.editor.title.new") }),
    /* @__PURE__ */ R("form", { onSubmit: (v) => {
      v.preventDefault(), y();
    }, className: "space-y-4 px-4 pt-1", children: [
      /* @__PURE__ */ d(te, { label: n("center.name"), htmlFor: "ce-name", required: !0, children: /* @__PURE__ */ d(pe, { id: "ce-name", value: o, onChange: (v) => s(v.target.value), required: !0 }) }),
      /* @__PURE__ */ d(te, { label: n("center.address"), htmlFor: "ce-address", children: /* @__PURE__ */ d(pe, { id: "ce-address", value: i, onChange: (v) => a(v.target.value) }) }),
      /* @__PURE__ */ d(te, { label: n("center.phone"), htmlFor: "ce-phone", children: /* @__PURE__ */ d(pe, { id: "ce-phone", type: "tel", value: c, onChange: (v) => l(v.target.value) }) }),
      /* @__PURE__ */ d(te, { label: n("center.email"), htmlFor: "ce-email", children: /* @__PURE__ */ d(pe, { id: "ce-email", type: "email", value: f, onChange: (v) => m(v.target.value) }) }),
      /* @__PURE__ */ d(te, { label: n("center.default_locale"), children: /* @__PURE__ */ d(
        Mn,
        {
          columns: 2,
          value: h,
          onChange: g,
          segments: [
            { value: "en", label: n("center.locale.en") },
            { value: "es", label: n("center.locale.es") }
          ]
        }
      ) }),
      x && /* @__PURE__ */ d("p", { role: "alert", className: "text-sm text-destructive", children: x }),
      /* @__PURE__ */ R("div", { className: "flex gap-2 pt-2", children: [
        /* @__PURE__ */ d(ie, { type: "button", variant: "outline", className: "flex-1", onClick: t, children: n("common.cancel") }),
        /* @__PURE__ */ d(ie, { type: "submit", className: "flex-1", disabled: w, children: n("common.save") })
      ] })
    ] })
  ] });
}
function yu({ centerId: e, initial: t, centers: n, onDone: r }) {
  var g;
  const o = fe(), s = Ae(), [i, a] = I((t == null ? void 0 : t.name) ?? ""), [c, l] = I((t == null ? void 0 : t.center_id) ?? e ?? ((g = n[0]) == null ? void 0 : g.id) ?? ""), [f, m] = I(!1);
  async function h() {
    m(!0);
    try {
      const w = (t == null ? void 0 : t.id) ?? (i.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "room");
      await s.run("room.create", { id: w, name: i, center_id: c }), r();
    } finally {
      m(!1);
    }
  }
  return /* @__PURE__ */ R("main", { className: "pb-24", children: [
    /* @__PURE__ */ d(he, { children: o(t ? "room.editor.title.edit" : "room.editor.title.new") }),
    /* @__PURE__ */ R("form", { onSubmit: (w) => {
      w.preventDefault(), h();
    }, className: "space-y-4 px-4 pt-1", children: [
      /* @__PURE__ */ d(te, { label: o("room.name"), htmlFor: "ro-name", required: !0, children: /* @__PURE__ */ d(pe, { id: "ro-name", value: i, onChange: (w) => a(w.target.value), required: !0 }) }),
      /* @__PURE__ */ d(te, { label: o("room.center"), children: /* @__PURE__ */ R($t, { value: c, onValueChange: l, children: [
        /* @__PURE__ */ d(ht, { children: /* @__PURE__ */ d(Wt, {}) }),
        /* @__PURE__ */ d(gt, { children: n.map((w) => /* @__PURE__ */ d(tt, { value: w.id, children: w.name }, w.id)) })
      ] }) }),
      /* @__PURE__ */ R("div", { className: "flex gap-2 pt-2", children: [
        /* @__PURE__ */ d(ie, { type: "button", variant: "outline", className: "flex-1", onClick: r, children: o("common.cancel") }),
        /* @__PURE__ */ d(ie, { type: "submit", className: "flex-1", disabled: f, children: o("common.save") })
      ] })
    ] })
  ] });
}
const Ws = u.forwardRef(
  ({ className: e, ...t }, n) => /* @__PURE__ */ d(
    "textarea",
    {
      ref: n,
      className: ne(
        "flex min-h-[80px] w-full rounded-xl border border-input bg-card px-4 py-2.5 text-base text-foreground shadow-sm transition-colors",
        "placeholder:text-muted-foreground",
        "focus-visible:outline-none focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40",
        "disabled:cursor-not-allowed disabled:opacity-50",
        e
      ),
      ...t
    }
  )
);
Ws.displayName = "Textarea";
var Jt = "Switch", [wu] = nt(Jt), [xu, qn] = wu(Jt);
function Cu(e) {
  const {
    __scopeSwitch: t,
    checked: n,
    children: r,
    defaultChecked: o,
    disabled: s,
    form: i,
    name: a,
    onCheckedChange: c,
    required: l,
    value: f = "on",
    // @ts-expect-error
    internal_do_not_use_render: m
  } = e, [h, g] = ft({
    prop: n,
    defaultProp: o ?? !1,
    onChange: c,
    caller: Jt
  }), [w, p] = u.useState(null), [x, b] = u.useState(null), y = u.useRef(!1), v = w ? !!i || !!w.closest("form") : (
    // We set this to true by default so that events bubble to forms without JS (SSR)
    !0
  ), S = {
    checked: h,
    setChecked: g,
    disabled: s,
    control: w,
    setControl: p,
    name: a,
    form: i,
    value: f,
    hasConsumerStoppedPropagationRef: y,
    required: l,
    defaultChecked: o,
    isFormControl: v,
    bubbleInput: x,
    setBubbleInput: b
  };
  return /* @__PURE__ */ d(xu, { scope: t, ...S, children: Su(m) ? m(S) : r });
}
var Bs = "SwitchTrigger", Vs = u.forwardRef(
  ({ __scopeSwitch: e, onClick: t, ...n }, r) => {
    const {
      control: o,
      form: s,
      value: i,
      disabled: a,
      checked: c,
      required: l,
      setControl: f,
      setChecked: m,
      hasConsumerStoppedPropagationRef: h,
      isFormControl: g,
      bubbleInput: w
    } = qn(Bs, e), p = re(r, f), x = u.useRef(c);
    return u.useEffect(() => {
      const b = s ? o == null ? void 0 : o.ownerDocument.getElementById(s) : o == null ? void 0 : o.form;
      if (b instanceof HTMLFormElement) {
        const y = () => m(x.current);
        return b.addEventListener("reset", y), () => b.removeEventListener("reset", y);
      }
    }, [o, s, m]), /* @__PURE__ */ d(
      q.button,
      {
        type: "button",
        role: "switch",
        "aria-checked": c,
        "aria-required": l,
        "data-state": Ks(c),
        "data-disabled": a ? "" : void 0,
        disabled: a,
        value: i,
        ...n,
        ref: p,
        onClick: Y(t, (b) => {
          m((y) => !y), w && g && (h.current = b.isPropagationStopped(), h.current || b.stopPropagation());
        })
      }
    );
  }
);
Vs.displayName = Bs;
var Yn = u.forwardRef(
  (e, t) => {
    const {
      __scopeSwitch: n,
      name: r,
      checked: o,
      defaultChecked: s,
      required: i,
      disabled: a,
      value: c,
      onCheckedChange: l,
      form: f,
      ...m
    } = e;
    return /* @__PURE__ */ d(
      Cu,
      {
        __scopeSwitch: n,
        checked: o,
        defaultChecked: s,
        disabled: a,
        required: i,
        onCheckedChange: l,
        name: r,
        form: f,
        value: c,
        internal_do_not_use_render: ({ isFormControl: h }) => /* @__PURE__ */ R(Bt, { children: [
          /* @__PURE__ */ d(
            Vs,
            {
              ...m,
              ref: t,
              __scopeSwitch: n
            }
          ),
          h && /* @__PURE__ */ d(
            Gs,
            {
              __scopeSwitch: n
            }
          )
        ] })
      }
    );
  }
);
Yn.displayName = Jt;
var Hs = "SwitchThumb", Us = u.forwardRef(
  (e, t) => {
    const { __scopeSwitch: n, ...r } = e, o = qn(Hs, n);
    return /* @__PURE__ */ d(
      q.span,
      {
        "data-state": Ks(o.checked),
        "data-disabled": o.disabled ? "" : void 0,
        ...r,
        ref: t
      }
    );
  }
);
Us.displayName = Hs;
var js = "SwitchBubbleInput", Gs = u.forwardRef(
  ({ __scopeSwitch: e, ...t }, n) => {
    const {
      control: r,
      hasConsumerStoppedPropagationRef: o,
      checked: s,
      defaultChecked: i,
      required: a,
      disabled: c,
      name: l,
      value: f,
      form: m,
      bubbleInput: h,
      setBubbleInput: g
    } = qn(js, e), w = re(n, g), p = os(s), x = Ko(r);
    u.useEffect(() => {
      const y = h;
      if (!y) return;
      const v = window.HTMLInputElement.prototype, E = Object.getOwnPropertyDescriptor(
        v,
        "checked"
      ).set, C = !o.current;
      if (p !== s && E) {
        const k = new Event("click", { bubbles: C });
        E.call(y, s), y.dispatchEvent(k);
      }
    }, [h, p, s, o]);
    const b = u.useRef(s);
    return /* @__PURE__ */ d(
      q.input,
      {
        type: "checkbox",
        "aria-hidden": !0,
        defaultChecked: i ?? b.current,
        required: a,
        disabled: c,
        name: l,
        value: f,
        form: m,
        ...t,
        tabIndex: -1,
        ref: w,
        style: {
          ...t.style,
          ...x,
          position: "absolute",
          pointerEvents: "none",
          opacity: 0,
          margin: 0,
          // We transform because the input is absolutely positioned but we have
          // rendered it **after** the button. This pulls it back to sit on top
          // of the button.
          transform: "translateX(-100%)"
        }
      }
    );
  }
);
Gs.displayName = js;
function Su(e) {
  return typeof e == "function";
}
function Ks(e) {
  return e ? "checked" : "unchecked";
}
const qs = u.forwardRef(({ className: e, ...t }, n) => /* @__PURE__ */ d(
  Yn,
  {
    className: ne(
      "peer inline-flex h-7 w-[46px] shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors",
      "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
      "disabled:cursor-not-allowed disabled:opacity-50",
      "data-[state=checked]:bg-primary data-[state=unchecked]:bg-muted",
      e
    ),
    ...t,
    ref: n,
    children: /* @__PURE__ */ d(
      Us,
      {
        className: ne(
          "pointer-events-none block h-6 w-6 rounded-full bg-background shadow-lg ring-0 transition-transform",
          "data-[state=checked]:translate-x-[18px] data-[state=unchecked]:translate-x-0"
        )
      }
    )
  }
));
qs.displayName = Yn.displayName;
function Ys({ embedded: e } = {}) {
  const t = fe(), n = Ae(), [r, o] = I(null), [s, i] = I([]), [a, c] = I(null), [l, f] = I(!1);
  async function m() {
    const [h, g] = await Promise.all([n.list("child"), n.list("room")]);
    o(h.filter((w) => !w.archived)), i(g);
  }
  return Je(() => {
    m().catch(() => {
    });
  }, []), a || l ? /* @__PURE__ */ d(Nu, { initial: a, rooms: s, onDone: () => {
    c(null), f(!1), m();
  } }) : /* @__PURE__ */ R("main", { className: e ? "" : "pb-24", children: [
    !e && /* @__PURE__ */ d(
      he,
      {
        trailing: /* @__PURE__ */ R(ie, { size: "pill", onClick: () => f(!0), children: [
          /* @__PURE__ */ d(et, {}),
          " ",
          t("common.add")
        ] }),
        children: t("nav.children")
      }
    ),
    /* @__PURE__ */ R("div", { className: "px-4", children: [
      e && /* @__PURE__ */ R(ie, { className: "mb-4 w-full", onClick: () => f(!0), children: [
        /* @__PURE__ */ d(et, {}),
        " ",
        t("child.editor.title.new")
      ] }),
      r === null ? /* @__PURE__ */ d("ul", { className: "space-y-2", "aria-busy": "true", children: [0, 1, 2].map((h) => /* @__PURE__ */ d("li", { className: "h-[70px] animate-pulse rounded-2xl bg-muted" }, h)) }) : r.length ? /* @__PURE__ */ d("ul", { className: "space-y-2", children: r.map((h) => {
        const g = h.allergies && h.allergies.length > 0;
        return /* @__PURE__ */ d("li", { children: /* @__PURE__ */ R(
          "button",
          {
            onClick: () => c(h),
            className: "flex w-full items-center gap-3 rounded-2xl border border-border bg-card p-4 text-left shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            children: [
              /* @__PURE__ */ R("span", { className: "min-w-0 flex-1", children: [
                /* @__PURE__ */ d("span", { className: "block truncate text-base font-semibold text-foreground", children: h.name }),
                /* @__PURE__ */ d("span", { className: "block text-[13px] text-muted-foreground", children: h.dob })
              ] }),
              g && /* @__PURE__ */ R("span", { className: "inline-flex items-center gap-1 rounded-full bg-destructive/10 px-2.5 py-1 text-xs font-semibold text-destructive", children: [
                /* @__PURE__ */ d(Mi, { className: "size-3.5", "aria-hidden": !0 }),
                " ",
                h.allergies.length
              ] }),
              /* @__PURE__ */ d(Pn, { className: "size-5 shrink-0 text-muted-foreground", "aria-hidden": !0 })
            ]
          }
        ) }, h.id);
      }) }) : /* @__PURE__ */ d("p", { className: "py-16 text-center text-[15px] text-muted-foreground", children: t("child.empty") })
    ] })
  ] });
}
function Nu({ initial: e, rooms: t, onDone: n }) {
  const r = fe(), o = Ae(), [s, i] = I((e == null ? void 0 : e.name) ?? ""), [a, c] = I((e == null ? void 0 : e.dob) ?? ""), [l, f] = I((e == null ? void 0 : e.room_id) ?? ""), [m, h] = I(((e == null ? void 0 : e.allergies) ?? []).join(", ")), [g, w] = I((e == null ? void 0 : e.medical_notes) ?? ""), [p, x] = I((e == null ? void 0 : e.photo_consent) ?? !1), [b, y] = I(!1), [v, S] = I(null);
  async function E() {
    y(!0), S(null);
    try {
      const C = (e == null ? void 0 : e.id) ?? (s.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "child"), k = m.split(",").map((L) => L.trim()).filter(Boolean), N = {
        id: C,
        name: s,
        dob: a,
        room_id: l || void 0,
        allergies: k,
        medical_notes: g || void 0,
        photo_consent: p
      };
      await o.run(e ? "child.update" : "child.create", N), n();
    } catch (C) {
      S(C.message ?? r("common.error_generic"));
    } finally {
      y(!1);
    }
  }
  return /* @__PURE__ */ R("main", { className: "pb-24", children: [
    /* @__PURE__ */ d(he, { children: r(e ? "child.editor.title.edit" : "child.editor.title.new") }),
    /* @__PURE__ */ R("form", { onSubmit: (C) => {
      C.preventDefault(), E();
    }, className: "space-y-6 px-4 pt-1", children: [
      /* @__PURE__ */ R("div", { className: "space-y-4", children: [
        /* @__PURE__ */ d(te, { label: r("child.name"), htmlFor: "c-name", required: !0, children: /* @__PURE__ */ d(pe, { id: "c-name", value: s, onChange: (C) => i(C.target.value), required: !0 }) }),
        /* @__PURE__ */ d(te, { label: r("child.dob"), htmlFor: "c-dob", required: !0, children: /* @__PURE__ */ d(pe, { id: "c-dob", type: "date", value: a, onChange: (C) => c(C.target.value), required: !0 }) }),
        /* @__PURE__ */ d(te, { label: r("child.room"), children: /* @__PURE__ */ R($t, { value: l || "none", onValueChange: (C) => f(C === "none" ? "" : C), children: [
          /* @__PURE__ */ d(ht, { children: /* @__PURE__ */ d(Wt, { placeholder: "—" }) }),
          /* @__PURE__ */ R(gt, { children: [
            /* @__PURE__ */ d(tt, { value: "none", children: "—" }),
            t.map((C) => /* @__PURE__ */ d(tt, { value: C.id, children: C.name }, C.id))
          ] })
        ] }) })
      ] }),
      /* @__PURE__ */ R(Or, { title: r("child.editor.safety"), hint: r("child.editor.safety_help"), children: [
        /* @__PURE__ */ d(te, { label: r("child.allergies"), htmlFor: "c-allergies", required: !0, hint: r("child.required.allergies_hint"), children: /* @__PURE__ */ d(pe, { id: "c-allergies", value: m, onChange: (C) => h(C.target.value), placeholder: "peanuts, dairy" }) }),
        /* @__PURE__ */ d(te, { label: r("child.medical_notes"), htmlFor: "c-medical", children: /* @__PURE__ */ d(Ws, { id: "c-medical", value: g, onChange: (C) => w(C.target.value), rows: 3 }) })
      ] }),
      /* @__PURE__ */ d(Or, { title: r("child.editor.consent"), children: /* @__PURE__ */ R("label", { className: "flex items-center justify-between gap-3 rounded-2xl border border-border bg-card p-4 shadow-sm", children: [
        /* @__PURE__ */ d("span", { className: "text-sm text-foreground", children: r("child.photo_consent") }),
        /* @__PURE__ */ d(qs, { checked: p, onCheckedChange: x })
      ] }) }),
      v && /* @__PURE__ */ d("p", { role: "alert", className: "text-sm text-destructive", children: v }),
      /* @__PURE__ */ R("div", { className: "flex gap-2 pt-2", children: [
        /* @__PURE__ */ d(ie, { type: "button", variant: "outline", className: "flex-1", onClick: n, children: r("common.cancel") }),
        /* @__PURE__ */ d(ie, { type: "submit", className: "flex-1", disabled: b, children: r("common.save") })
      ] })
    ] })
  ] });
}
function Or({ title: e, hint: t, children: n }) {
  return /* @__PURE__ */ R("section", { className: "space-y-3", children: [
    /* @__PURE__ */ R("div", { children: [
      /* @__PURE__ */ d("h2", { className: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", children: e }),
      t && /* @__PURE__ */ d("p", { className: "pt-1 text-xs leading-relaxed text-muted-foreground", children: t })
    ] }),
    n
  ] });
}
const Eu = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"], Mr = ["enrolled", "waitlist", "withdrawn"];
function ku({ embedded: e } = {}) {
  var b;
  const t = fe(), n = Ae(), [r, o] = I(null), [s, i] = I([]), [a, c] = I([]), [l, f] = I(null), [m, h] = I(!1), [g, w] = I(null);
  async function p() {
    const [y, v, S] = await Promise.all([
      n.list("enrollment"),
      n.list("room"),
      n.list("child")
    ]);
    o(y), i(v), c(S);
  }
  if (Je(() => {
    p().catch(() => {
    });
  }, []), l || m)
    return /* @__PURE__ */ d(Ru, { initial: l, rooms: s, children: a, onDone: () => {
      f(null), h(!1), p();
    } });
  if (g) {
    const y = (r ?? []).filter((v) => v.room_id === g && v.status === "waitlist").sort((v, S) => (v.waitlist_seq ?? 0) - (S.waitlist_seq ?? 0));
    return /* @__PURE__ */ R("main", { className: "pb-24", children: [
      /* @__PURE__ */ d(he, { children: t("enrollment.waitlist.title") }),
      /* @__PURE__ */ R("div", { className: "px-4 pt-1", children: [
        /* @__PURE__ */ R(ie, { variant: "ghost", size: "sm", className: "mb-3 -ml-2 text-primary", onClick: () => w(null), children: [
          /* @__PURE__ */ d(Ni, {}),
          " ",
          t("common.back")
        ] }),
        /* @__PURE__ */ d("h2", { className: "pb-3 text-base font-semibold text-foreground", children: (b = s.find((v) => v.id === g)) == null ? void 0 : b.name }),
        y.length ? /* @__PURE__ */ d("ol", { className: "space-y-2", children: y.map((v, S) => {
          const E = a.find((C) => C.id === v.child_id);
          return /* @__PURE__ */ R("li", { className: "flex items-center gap-3 rounded-2xl border border-border bg-card p-4 shadow-sm", children: [
            /* @__PURE__ */ d("span", { className: "flex size-8 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold text-primary", children: S + 1 }),
            /* @__PURE__ */ d("span", { className: "flex-1 font-medium text-foreground", children: (E == null ? void 0 : E.name) ?? v.child_id }),
            /* @__PURE__ */ d("span", { className: "text-xs text-muted-foreground", children: v.waitlist_seq ? t("enrollment.position", { position: String(v.waitlist_seq) }) : "" })
          ] }, v.id ?? `${v.child_id}-${S}`);
        }) }) : /* @__PURE__ */ d("p", { className: "py-16 text-center text-[15px] text-muted-foreground", children: t("enrollment.waitlist_empty") })
      ] })
    ] });
  }
  const x = /* @__PURE__ */ new Map();
  for (const y of r ?? []) {
    const v = x.get(y.room_id) ?? [];
    v.push(y), x.set(y.room_id, v);
  }
  return /* @__PURE__ */ R("main", { className: e ? "" : "pb-24", children: [
    !e && /* @__PURE__ */ d(he, { children: t("enrollment.list.title") }),
    /* @__PURE__ */ R("div", { className: "px-4 pt-1", children: [
      /* @__PURE__ */ R(ie, { className: "mb-4 w-full", onClick: () => h(!0), children: [
        /* @__PURE__ */ d(et, {}),
        " ",
        t("enrollment.editor.title.new")
      ] }),
      s.map((y) => {
        const v = (x.get(y.id) ?? []).filter((C) => C.status !== "withdrawn"), S = v.filter((C) => C.status === "waitlist").sort((C, k) => (C.waitlist_seq ?? 0) - (k.waitlist_seq ?? 0)), E = v.filter((C) => C.status === "enrolled");
        return /* @__PURE__ */ R("section", { className: "mb-4 rounded-2xl border border-border bg-card p-4 shadow-sm", children: [
          /* @__PURE__ */ R("header", { className: "flex items-baseline justify-between", children: [
            /* @__PURE__ */ d("h2", { className: "text-base font-semibold text-foreground", children: y.name }),
            /* @__PURE__ */ R("span", { className: "text-xs text-muted-foreground", children: [
              E.length,
              " ",
              t("enrollment.status.enrolled")
            ] })
          ] }),
          S.length > 0 && /* @__PURE__ */ R("button", { onClick: () => w(y.id), className: "mt-3 flex w-full items-center justify-between rounded-xl bg-muted px-3 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-accent", children: [
            /* @__PURE__ */ R("span", { children: [
              S.length,
              " ",
              t("enrollment.status.waitlist")
            ] }),
            /* @__PURE__ */ d(Pn, { className: "size-4 text-muted-foreground", "aria-hidden": !0 })
          ] }),
          E.length > 0 && /* @__PURE__ */ d("ul", { className: "mt-3 space-y-1", children: E.map((C) => {
            const k = a.find((N) => N.id === C.child_id);
            return /* @__PURE__ */ d("li", { children: /* @__PURE__ */ R("button", { onClick: () => f(C), className: "flex w-full items-center gap-2 rounded-lg px-1 py-1.5 text-left text-sm text-foreground transition-colors hover:bg-accent", children: [
              /* @__PURE__ */ d("span", { className: "font-medium", children: (k == null ? void 0 : k.name) ?? C.child_id }),
              C.schedule && C.schedule.length > 0 && /* @__PURE__ */ d("span", { className: "text-xs text-muted-foreground", children: C.schedule.map((N) => t(`enrollment.day.${N}`)).join(" ") })
            ] }) }, C.id ?? `${C.child_id}-${C.room_id}`);
          }) })
        ] }, y.id);
      })
    ] })
  ] });
}
function Ru({ initial: e, rooms: t, children: n, onDone: r }) {
  var S, E;
  const o = fe(), s = Ae(), [i, a] = I((e == null ? void 0 : e.child_id) ?? ((S = n[0]) == null ? void 0 : S.id) ?? ""), [c, l] = I((e == null ? void 0 : e.room_id) ?? ((E = t[0]) == null ? void 0 : E.id) ?? ""), [f, m] = I((e == null ? void 0 : e.status) ?? "enrolled"), [h, g] = I((e == null ? void 0 : e.schedule) ?? []), [w, p] = I((e == null ? void 0 : e.start_date) ?? ""), [x, b] = I(!1);
  async function y() {
    b(!0);
    try {
      e != null && e.child_id && e.room_id ? await s.run("enrollment.update", {
        child_id: e.child_id,
        room_id: e.room_id,
        status: f,
        schedule: h,
        start_date: w || void 0
      }) : await s.run("enrollment.create", {
        child_id: i,
        room_id: c,
        status: f,
        schedule: h,
        start_date: w || void 0
      }), r();
    } finally {
      b(!1);
    }
  }
  function v(C) {
    g((k) => k.includes(C) ? k.filter((N) => N !== C) : [...k, C]);
  }
  return /* @__PURE__ */ R("main", { className: "pb-24", children: [
    /* @__PURE__ */ d(he, { children: o(e ? "enrollment.editor.title.edit" : "enrollment.editor.title.new") }),
    /* @__PURE__ */ R("form", { onSubmit: (C) => {
      C.preventDefault(), y();
    }, className: "space-y-4 px-4 pt-1", children: [
      /* @__PURE__ */ d(te, { label: o("enrollment.child"), children: /* @__PURE__ */ R($t, { value: i, onValueChange: a, disabled: !!e, children: [
        /* @__PURE__ */ d(ht, { children: /* @__PURE__ */ d(Wt, {}) }),
        /* @__PURE__ */ d(gt, { children: n.map((C) => /* @__PURE__ */ d(tt, { value: C.id, children: C.name }, C.id)) })
      ] }) }),
      /* @__PURE__ */ d(te, { label: o("enrollment.room"), children: /* @__PURE__ */ R($t, { value: c, onValueChange: l, disabled: !!e, children: [
        /* @__PURE__ */ d(ht, { children: /* @__PURE__ */ d(Wt, {}) }),
        /* @__PURE__ */ d(gt, { children: t.map((C) => /* @__PURE__ */ d(tt, { value: C.id, children: C.name }, C.id)) })
      ] }) }),
      /* @__PURE__ */ d(te, { label: o("enrollment.status"), children: /* @__PURE__ */ d(
        Mn,
        {
          columns: Mr.length,
          value: f,
          onChange: m,
          segments: Mr.map((C) => ({ value: C, label: o(`enrollment.status.${C}`) }))
        }
      ) }),
      /* @__PURE__ */ d(te, { label: o("enrollment.schedule"), children: /* @__PURE__ */ d("div", { className: "grid grid-cols-7 gap-1.5", children: Eu.map((C) => {
        const k = h.includes(C);
        return /* @__PURE__ */ d(
          "button",
          {
            type: "button",
            "aria-pressed": k,
            onClick: () => v(C),
            className: ne(
              "rounded-xl border py-2.5 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              k ? "border-primary bg-primary text-primary-foreground" : "border-border bg-card text-muted-foreground hover:text-foreground"
            ),
            children: o(`enrollment.day.${C}`)
          },
          C
        );
      }) }) }),
      /* @__PURE__ */ d(te, { label: o("enrollment.start_date"), htmlFor: "en-start", children: /* @__PURE__ */ d(pe, { id: "en-start", type: "date", value: w, onChange: (C) => p(C.target.value) }) }),
      /* @__PURE__ */ R("div", { className: "flex gap-2 pt-2", children: [
        /* @__PURE__ */ d(ie, { type: "button", variant: "outline", className: "flex-1", onClick: r, children: o("common.cancel") }),
        /* @__PURE__ */ d(ie, { type: "submit", className: "flex-1", disabled: x, children: o("common.save") })
      ] })
    ] })
  ] });
}
function Pu({ embedded: e } = {}) {
  const t = fe(), n = Ae(), [r, o] = I(null), [s, i] = I(null), [a, c] = I(!1);
  async function l() {
    o(await n.list("guardian"));
  }
  return Je(() => {
    l().catch(() => {
    });
  }, []), s || a ? /* @__PURE__ */ d(Au, { initial: s, onDone: () => {
    i(null), c(!1), l();
  } }) : /* @__PURE__ */ R("main", { className: e ? "" : "pb-24", children: [
    !e && /* @__PURE__ */ d(he, { children: t("guardian.list.title") }),
    /* @__PURE__ */ R("div", { className: "px-4 pt-1", children: [
      /* @__PURE__ */ R(ie, { className: "mb-4 w-full", onClick: () => c(!0), children: [
        /* @__PURE__ */ d(et, {}),
        " ",
        t("guardian.editor.title.new")
      ] }),
      r === null ? /* @__PURE__ */ d("ul", { className: "space-y-2", "aria-busy": "true", children: [0, 1].map((f) => /* @__PURE__ */ d("li", { className: "h-[70px] animate-pulse rounded-2xl bg-muted" }, f)) }) : r.length ? /* @__PURE__ */ d("ul", { className: "space-y-2", children: r.map((f) => /* @__PURE__ */ d("li", { children: /* @__PURE__ */ R("button", { onClick: () => i(f), className: "flex w-full items-center gap-3 rounded-2xl border border-border bg-card p-4 text-left shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring", children: [
        /* @__PURE__ */ R("span", { className: "min-w-0 flex-1", children: [
          /* @__PURE__ */ d("span", { className: "block truncate text-base font-semibold text-foreground", children: f.name }),
          /* @__PURE__ */ d("span", { className: "block truncate text-[13px] text-muted-foreground", children: f.email })
        ] }),
        !f.sub && /* @__PURE__ */ d("span", { className: "rounded-full bg-primary/10 px-2.5 py-1 text-xs font-semibold text-primary", children: t("guardian.invite_pending") }),
        /* @__PURE__ */ d(Pn, { className: "size-5 shrink-0 text-muted-foreground", "aria-hidden": !0 })
      ] }) }, f.id)) }) : /* @__PURE__ */ d("p", { className: "py-16 text-center text-[15px] text-muted-foreground", children: t("guardian.empty") })
    ] })
  ] });
}
function Au({ initial: e, onDone: t }) {
  const n = fe(), r = Ae(), [o, s] = I((e == null ? void 0 : e.name) ?? ""), [i, a] = I((e == null ? void 0 : e.email) ?? ""), [c, l] = I((e == null ? void 0 : e.phone) ?? ""), [f, m] = I((e == null ? void 0 : e.locale) ?? "en"), [h, g] = I(!1), [w, p] = I(null);
  async function x() {
    g(!0), p(null);
    try {
      const b = (i.split("@")[0] ?? "guardian").toLowerCase().replace(/[^a-z0-9]+/g, "-"), y = (e == null ? void 0 : e.id) ?? b;
      await r.run("guardian.create", { id: y, name: o, email: i, phone: c, locale: f }), t();
    } catch {
      p(n("common.error_generic"));
    } finally {
      g(!1);
    }
  }
  return /* @__PURE__ */ R("main", { className: "pb-24", children: [
    /* @__PURE__ */ d(he, { children: n(e ? "guardian.editor.title.edit" : "guardian.editor.title.new") }),
    /* @__PURE__ */ R("form", { onSubmit: (b) => {
      b.preventDefault(), x();
    }, className: "space-y-4 px-4 pt-1", children: [
      /* @__PURE__ */ d(te, { label: n("guardian.name"), htmlFor: "g-name", required: !0, children: /* @__PURE__ */ d(pe, { id: "g-name", value: o, onChange: (b) => s(b.target.value), required: !0 }) }),
      /* @__PURE__ */ d(te, { label: n("guardian.email"), htmlFor: "g-email", required: !0, children: /* @__PURE__ */ d(pe, { id: "g-email", type: "email", value: i, onChange: (b) => a(b.target.value), required: !0 }) }),
      /* @__PURE__ */ d(te, { label: n("guardian.phone"), htmlFor: "g-phone", children: /* @__PURE__ */ d(pe, { id: "g-phone", type: "tel", value: c, onChange: (b) => l(b.target.value) }) }),
      /* @__PURE__ */ d(te, { label: n("guardian.locale"), children: /* @__PURE__ */ d(
        Mn,
        {
          columns: 2,
          value: f,
          onChange: m,
          segments: [
            { value: "en", label: n("center.locale.en") },
            { value: "es", label: n("center.locale.es") }
          ]
        }
      ) }),
      w && /* @__PURE__ */ d("p", { role: "alert", className: "text-sm text-destructive", children: w }),
      /* @__PURE__ */ R("div", { className: "flex gap-2 pt-2", children: [
        /* @__PURE__ */ d(ie, { type: "button", variant: "outline", className: "flex-1", onClick: t, children: n("common.cancel") }),
        /* @__PURE__ */ d(ie, { type: "submit", className: "flex-1", disabled: h, children: n("common.save") })
      ] })
    ] })
  ] });
}
function Tu() {
  const e = fe(), [t, n] = I("schools"), r = [
    { key: "schools", label: e("admin.schools") },
    { key: "children", label: e("nav.children") },
    { key: "enrollment", label: e("admin.enrollment") },
    { key: "guardians", label: e("admin.guardians") }
  ];
  return /* @__PURE__ */ R("div", { className: "pb-24", children: [
    /* @__PURE__ */ d(he, { children: e("admin.title") }),
    /* @__PURE__ */ R(bc, { value: t, onValueChange: (o) => n(o), children: [
      /* @__PURE__ */ d("div", { className: "sticky top-12 z-10 bg-background/70 px-4 pb-2 pt-1 backdrop-blur-xl", children: /* @__PURE__ */ d(Eo, { className: "w-full", children: r.map((o) => /* @__PURE__ */ d(ko, { value: o.key, className: "flex-1", children: o.label }, o.key)) }) }),
      /* @__PURE__ */ d(lt, { value: "schools", children: /* @__PURE__ */ d(vu, { embedded: !0 }) }),
      /* @__PURE__ */ d(lt, { value: "children", children: /* @__PURE__ */ d(Ys, { embedded: !0 }) }),
      /* @__PURE__ */ d(lt, { value: "enrollment", children: /* @__PURE__ */ d(ku, { embedded: !0 }) }),
      /* @__PURE__ */ d(lt, { value: "guardians", children: /* @__PURE__ */ d(Pu, { embedded: !0 }) })
    ] })
  ] });
}
function _u() {
  const e = fe(), t = Wr(), [n, r] = I("today"), o = (t == null ? void 0 : t.role) === "admin";
  return /* @__PURE__ */ R("div", { children: [
    n === "today" && /* @__PURE__ */ R("main", { className: "pb-24", children: [
      /* @__PURE__ */ d(he, { children: e("app.title") }),
      /* @__PURE__ */ R("div", { className: "px-4", children: [
        t && /* @__PURE__ */ R("p", { className: "text-[13px] capitalize text-muted-foreground", children: [
          t.role,
          " · ",
          t.workspaceId
        ] }),
        /* @__PURE__ */ d("div", { className: "flex flex-col items-center gap-2 py-20 text-center", children: /* @__PURE__ */ d("p", { className: "text-[15px] text-muted-foreground", children: e("feed.empty") }) })
      ] })
    ] }),
    n === "children" && /* @__PURE__ */ d(Ys, {}),
    n === "admin" && o && /* @__PURE__ */ d(Tu, {}),
    /* @__PURE__ */ d(Pa, { active: n, onChange: r, showAdmin: o })
  ] });
}
const Xs = u.forwardRef(
  ({ className: e, ...t }, n) => /* @__PURE__ */ d(
    "div",
    {
      ref: n,
      className: ne(
        "rounded-2xl border border-border bg-card text-card-foreground shadow-[0_1px_3px_0_hsl(var(--foreground)/0.04),0_8px_24px_-12px_hsl(var(--foreground)/0.10)]",
        e
      ),
      ...t
    }
  )
);
Xs.displayName = "Card";
const Zs = u.forwardRef(
  ({ className: e, ...t }, n) => /* @__PURE__ */ d("div", { ref: n, className: ne("flex flex-col space-y-1.5 p-4", e), ...t })
);
Zs.displayName = "CardHeader";
const Qs = u.forwardRef(
  ({ className: e, ...t }, n) => /* @__PURE__ */ d("div", { ref: n, className: ne("font-semibold leading-none tracking-tight", e), ...t })
);
Qs.displayName = "CardTitle";
const Js = u.forwardRef(
  ({ className: e, ...t }, n) => /* @__PURE__ */ d("div", { ref: n, className: ne("p-4 pt-0", e), ...t })
);
Js.displayName = "CardContent";
function Iu({ childId: e }) {
  const t = fe();
  return /* @__PURE__ */ R(Xs, { "data-child": e, children: [
    /* @__PURE__ */ d(Zs, { className: "p-3 pb-1", children: /* @__PURE__ */ d(Qs, { className: "text-sm", children: t("menu.today") }) }),
    /* @__PURE__ */ d(Js, { className: "p-3 pt-0 text-sm text-muted-foreground", children: t("menu.substitutions") })
  ] });
}
function Ou() {
  const e = fe();
  return /* @__PURE__ */ d("span", { className: "inline-flex items-center rounded-full border border-border bg-card px-2.5 py-0.5 text-xs font-medium text-foreground", children: e("attendance.checkIn") });
}
const Fu = di({
  id: "care",
  styles: ui,
  page: () => /* @__PURE__ */ d(en, { children: /* @__PURE__ */ d(_u, {}) }),
  widgets: {
    // Keyed by the manifest `[[widget]]` slug the shell passes; each tile owns
    // its own LocaleProvider so a widget mounted standalone still translates.
    "next-meal": (e) => {
      var t;
      return /* @__PURE__ */ d(en, { children: /* @__PURE__ */ d(Iu, { childId: ((t = e.binding) == null ? void 0 : t.childId) ?? "" }) });
    },
    "attendance-badge": () => /* @__PURE__ */ d(en, { children: /* @__PURE__ */ d(Ou, {}) })
  }
});
export {
  Fu as default
};
