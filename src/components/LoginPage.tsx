import { useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { useAuth } from "@/contexts/AuthContext";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Lock, AlertCircle } from "lucide-react";

export function LoginPage() {
  const { t } = useTranslation();
  const { login, error } = useAuth();
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [localError, setLocalError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    if (!password.trim()) {
      setLocalError(t("auth.passwordRequired", { defaultValue: "Please enter password" }));
      return;
    }

    setIsSubmitting(true);

    try {
      const success = await login(password);
      if (!success) {
        setPassword("");
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  const displayError = localError || error;

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <Card className="w-full max-w-sm">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-blue-500/10">
            <Lock className="h-6 w-6 text-blue-500" />
          </div>
          <CardTitle className="text-xl">CC Switch</CardTitle>
          <CardDescription>
            {t("auth.loginDescription", { defaultValue: "Enter password to continue" })}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="password">
                {t("auth.password", { defaultValue: "Password" })}
              </Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder={t("auth.passwordPlaceholder", { defaultValue: "Enter password" })}
                disabled={isSubmitting}
                autoFocus
                autoComplete="current-password"
              />
            </div>

            {displayError && (
              <div className="flex items-center gap-2 text-sm text-red-500">
                <AlertCircle className="h-4 w-4 flex-shrink-0" />
                <span>{displayError}</span>
              </div>
            )}

            <Button type="submit" className="w-full" disabled={isSubmitting}>
              {isSubmitting
                ? t("auth.loggingIn", { defaultValue: "Logging in..." })
                : t("auth.login", { defaultValue: "Login" })}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
