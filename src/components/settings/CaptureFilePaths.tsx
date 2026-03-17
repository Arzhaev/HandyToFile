import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { useSettings } from "../../hooks/useSettings";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";

type PathKey = "notes_path" | "tasks_path" | "ideas_path" | "shopping_path";

const PATH_CONFIG: Array<{
  key: PathKey;
  command: string;
  title: string;
  description: string;
}> = [
  {
    key: "notes_path",
    command: "change_notes_path_setting",
    title: "Notes file",
    description: "Markdown file used by the quick note capture action.",
  },
  {
    key: "tasks_path",
    command: "change_tasks_path_setting",
    title: "Tasks file",
    description: "Markdown file used by the quick task capture action.",
  },
  {
    key: "ideas_path",
    command: "change_ideas_path_setting",
    title: "Ideas file",
    description: "Markdown file used by the quick idea capture action.",
  },
  {
    key: "shopping_path",
    command: "change_shopping_path_setting",
    title: "Shopping file",
    description: "Markdown file used by the quick shopping capture action.",
  },
];

interface CaptureFilePathsProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const CaptureFilePaths: React.FC<CaptureFilePathsProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { settings, refreshSettings } = useSettings();
  const [drafts, setDrafts] = useState<Record<PathKey, string>>({
    notes_path: "",
    tasks_path: "",
    ideas_path: "",
    shopping_path: "",
  });
  const [savingKey, setSavingKey] = useState<PathKey | null>(null);

  useEffect(() => {
    if (!settings) return;
    setDrafts({
      notes_path: settings.notes_path ?? "",
      tasks_path: settings.tasks_path ?? "",
      ideas_path: settings.ideas_path ?? "",
      shopping_path: settings.shopping_path ?? "",
    });
  }, [settings]);

  const savePath = async (key: PathKey, command: string) => {
    const value = drafts[key].trim();
    setSavingKey(key);
    try {
      await invoke(command, { path: value });
      await refreshSettings();
      toast.success(
        t("settings.general.capturePaths.saved", {
          defaultValue: "Capture path saved",
        }),
      );
    } catch (error) {
      console.error(`Failed to save ${key}:`, error);
      toast.error(
        t("settings.general.capturePaths.saveError", {
          defaultValue: "Failed to save capture path",
        }),
      );
    } finally {
      setSavingKey(null);
    }
  };

  return (
    <div className="space-y-2">
      {PATH_CONFIG.map((item) => (
        <SettingContainer
          key={item.key}
          title={t(`settings.general.capturePaths.${item.key}.title`, {
            defaultValue: item.title,
          })}
          description={t(`settings.general.capturePaths.${item.key}.description`, {
            defaultValue: item.description,
          })}
          descriptionMode={descriptionMode}
          grouped={grouped}
          layout="stacked"
        >
          <div className="flex gap-2 w-full max-w-2xl">
            <Input
              value={drafts[item.key]}
              onChange={(event) =>
                setDrafts((prev) => ({ ...prev, [item.key]: event.target.value }))
              }
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.preventDefault();
                  void savePath(item.key, item.command);
                }
              }}
              placeholder={item.title}
              className="flex-1"
            />
            <Button
              onClick={() => void savePath(item.key, item.command)}
              disabled={savingKey === item.key || !drafts[item.key].trim()}
              variant="primary"
              size="md"
            >
              {t("common.save", { defaultValue: "Save" })}
            </Button>
          </div>
        </SettingContainer>
      ))}
    </div>
  );
};
