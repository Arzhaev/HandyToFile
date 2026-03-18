import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useSettings } from "../../hooks/useSettings";
import { SettingContainer } from "../ui/SettingContainer";
import { Select } from "../ui/Select";
import type { ActionMeta } from "react-select";

interface RecognitionProfileSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const RecognitionProfileSelector: React.FC<
  RecognitionProfileSelectorProps
> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const { t } = useTranslation();
  const { settings, refreshSettings } = useSettings();
  const [isSaving, setIsSaving] = useState(false);

  const profiles = settings?.recognition_profiles ?? [];
  const captureActions = settings?.capture_actions ?? [];

  // Determine current profile: use the first action's profile_id,
  // show "mixed" label if actions differ
  const firstProfileId = captureActions[0]?.profile_id ?? null;
  const allSame = captureActions.every((a) => a.profile_id === firstProfileId);
  const currentValue = allSame ? firstProfileId : null;

  const options = profiles.map((p) => ({
    value: p.id,
    label: p.name,
  }));

  const handleChange = async (
    value: string | null,
    _action: ActionMeta<{ value: string; label: string }>,
  ) => {
    if (!value) return;
    setIsSaving(true);
    try {
      await invoke("set_recognition_profile", { profileId: value });
      await refreshSettings();
      const label = options.find((o) => o.value === value)?.label ?? value;
      toast.success(
        t("settings.general.recognitionProfile.saved", {
          defaultValue: "Profile set to {{name}}",
          name: label,
        }),
      );
    } catch (error) {
      console.error("Failed to set recognition profile:", error);
      toast.error(
        t("settings.general.recognitionProfile.saveError", {
          defaultValue: "Failed to set recognition profile",
        }),
      );
    } finally {
      setIsSaving(false);
    }
  };

  if (profiles.length === 0) return null;

  return (
    <SettingContainer
      title={t("settings.general.recognitionProfile.title", {
        defaultValue: "Recognition Profile",
      })}
      description={t("settings.general.recognitionProfile.description", {
        defaultValue:
          "Language and processing style applied to all capture actions.",
      })}
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <Select
        value={currentValue}
        options={options}
        isLoading={isSaving}
        isClearable={false}
        placeholder={
          allSame
            ? undefined
            : t("settings.general.recognitionProfile.mixed", {
                defaultValue: "Mixed",
              })
        }
        onChange={handleChange}
        className="w-44"
      />
    </SettingContainer>
  );
};
