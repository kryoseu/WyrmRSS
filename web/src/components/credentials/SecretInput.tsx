import { type ComponentProps, useState } from "react";
import { TbEye, TbEyeOff } from "react-icons/tb";

/** Password input with a show/hide toggle. */
export function SecretInput(props: Omit<ComponentProps<"input">, "type">) {
  const [visible, setVisible] = useState(false);

  return (
    <div className="secret-input">
      <input {...props} type={visible ? "text" : "password"} />
      <button
        type="button"
        className="secret-input-toggle"
        onClick={() => setVisible((v) => !v)}
        title={visible ? "Hide" : "Show"}
        aria-label={visible ? "Hide secret" : "Show secret"}
      >
        {visible ? <TbEyeOff /> : <TbEye />}
      </button>
    </div>
  );
}
