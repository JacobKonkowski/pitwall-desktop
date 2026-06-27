import type { ReactNode } from "react";

interface Props {
  title: string;
  description?: string;
  defaultOpen?: boolean;
  children: ReactNode;
}

export function SettingsSection({ title, description, defaultOpen = true, children }: Props) {
  return (
    <details className="settings-section" open={defaultOpen}>
      <summary className="settings-section-title">{title}</summary>
      {description && <p className="muted small settings-section-desc">{description}</p>}
      <div className="settings-section-body">{children}</div>
    </details>
  );
}

interface RowProps {
  label: string;
  children: ReactNode;
  className?: string;
}

export function SettingsRow({ label, children, className = "" }: RowProps) {
  return (
    <label className={`settings-row ${className}`.trim()}>
      <span>{label}</span>
      {children}
    </label>
  );
}

interface ToggleProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function SettingsToggle({ label, checked, onChange }: ToggleProps) {
  return (
    <SettingsRow label={label} className="checkbox">
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
    </SettingsRow>
  );
}

interface TextProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export function SettingsTextInput({ label, value, onChange, placeholder }: TextProps) {
  return (
    <SettingsRow label={label}>
      <input value={value} placeholder={placeholder} onChange={(e) => onChange(e.target.value)} />
    </SettingsRow>
  );
}

interface SelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
}

export function SettingsSelect({ label, value, onChange, options }: SelectProps) {
  return (
    <SettingsRow label={label}>
      <select value={value} onChange={(e) => onChange(e.target.value)}>
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </SettingsRow>
  );
}

interface SliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  format?: (v: number) => string;
  onChange: (value: number) => void;
}

export function SettingsSlider({ label, value, min, max, step, format, onChange }: SliderProps) {
  const display = format ? format(value) : String(value);
  return (
    <SettingsRow label={`${label} (${display})`}>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
      />
    </SettingsRow>
  );
}
