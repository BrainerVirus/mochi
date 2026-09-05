import { Field, FieldContent, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

export function WarnPercentField({
  id,
  label,
  description,
  value,
  placeholder,
  onChange,
}: {
  id: string;
  label: string;
  description: string;
  value: number | undefined;
  placeholder?: string;
  onChange: (value: number | undefined) => void;
}) {
  return (
    <Field orientation="horizontal" className="items-center justify-between gap-3 py-2.5">
      <FieldContent className="min-w-0">
        <FieldLabel htmlFor={id} className="text-sm font-medium">
          {label}
        </FieldLabel>
        <FieldDescription className="text-[11px]">{description}</FieldDescription>
      </FieldContent>
      <Input
        id={id}
        type="text"
        inputMode="numeric"
        pattern="[0-9]*"
        className="h-7 w-20 shrink-0 tabular-nums"
        value={value ?? ""}
        placeholder={placeholder}
        onChange={(event) => {
          const raw = event.target.value.replace(/\D/g, "");
          if (raw === "") {
            onChange(undefined);
            return;
          }
          const parsed = Number.parseInt(raw, 10);
          if (Number.isFinite(parsed)) {
            onChange(Math.min(100, Math.max(1, parsed)));
          }
        }}
      />
    </Field>
  );
}
