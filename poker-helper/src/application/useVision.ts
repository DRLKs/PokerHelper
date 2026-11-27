import { useEffect, useState } from 'react';
import { listWindows, startSidecar } from '../infrastructure/pokerApi';

export const useVision = () => {
  const [visionEnabled, setVisionEnabled] = useState(false);
  const [availableWindows, setAvailableWindows] = useState<string[]>([]);
  const [selectedWindow, setSelectedWindow] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    startSidecar().catch(console.error);
  }, []);

  const toggleVision = async () => {
    if (visionEnabled) {
      setVisionEnabled(false);
      return;
    }
    try {
      setAvailableWindows(await listWindows());
      setVisionEnabled(true);
      setError(null);
    } catch (reason: unknown) {
      setError(`Failed to list windows: ${String(reason)}`);
    }
  };

  return { visionEnabled, toggleVision, availableWindows, selectedWindow, setSelectedWindow, error };
};
