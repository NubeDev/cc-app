import { useT } from "../hooks/useT";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";

interface Props { childId: string; }

export function NextMealWidget({ childId }: Props) {
  const t = useT();
  return (
    <Card data-child={childId}>
      <CardHeader className="p-3 pb-1">
        <CardTitle className="text-sm">{t("menu.today")}</CardTitle>
      </CardHeader>
      <CardContent className="p-3 pt-0 text-sm text-muted-foreground">{t("menu.substitutions")}</CardContent>
    </Card>
  );
}
