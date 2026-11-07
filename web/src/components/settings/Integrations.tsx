import { useSettings } from "../../hooks/useSettings";
import { RSSHubForm } from "../integrations/RSSHubForm";

export function Integrations() {
  const { data } = useSettings();
  if (!data) return null;
  return <RSSHubForm initialInstance={data.rsshub_instance ?? ""} />;
}
