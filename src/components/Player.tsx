// ArtPlayer + hls.js 播放器封装：进度回调 + 起播位置
import { useEffect, useRef } from "react";
import Artplayer from "artplayer";
import Hls from "hls.js";

interface Props {
  url: string;
  poster?: string;
  startPos?: number;
  /** 节流后的播放进度回调（秒） */
  onProgress?: (positionSec: number, durationSec: number) => void;
}

function playM3u8(video: HTMLVideoElement, url: string, art: Artplayer) {
  if (Hls.isSupported()) {
    if ((art as unknown as { hls?: Hls }).hls) {
      (art as unknown as { hls?: Hls }).hls?.destroy();
    }
    const hls = new Hls({ maxBufferLength: 30 });
    hls.loadSource(url);
    hls.attachMedia(video);
    (art as unknown as { hls?: Hls }).hls = hls;
    art.on("destroy", () => hls.destroy());
  } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
    video.src = url;
  } else {
    art.notice.show = "当前环境不支持 HLS 播放";
  }
}

export default function Player({ url, poster, startPos, onProgress }: Props) {
  const boxRef = useRef<HTMLDivElement>(null);
  const artRef = useRef<Artplayer | null>(null);
  const lastReport = useRef(0);
  const onProgressRef = useRef(onProgress);
  onProgressRef.current = onProgress;

  useEffect(() => {
    if (!boxRef.current || !url) return;
    const isHls = url.toLowerCase().includes(".m3u8");

    const art = new Artplayer({
      container: boxRef.current,
      url,
      poster,
      type: isHls ? "m3u8" : undefined,
      customType: isHls ? { m3u8: playM3u8 } : undefined,
      autoSize: false,
      autoMini: false,
      setting: true,
      playbackRate: true,
      aspectRatio: true,
      flip: true,
      fullscreen: true,
      fullscreenWeb: true,
      miniProgressBar: true,
      mutex: true,
      lang: "zh-cn",
    } as ConstructorParameters<typeof Artplayer>[0]);

    art.on("ready", () => {
      if (startPos && startPos > 5) {
        art.currentTime = startPos;
        art.notice.show = `已从 ${Math.floor(startPos / 60)} 分钟处继续播放`;
      }
    });

    art.on("video:timeupdate", () => {
      const now = Date.now();
      if (now - lastReport.current > 8000) {
        lastReport.current = now;
        onProgressRef.current?.(art.currentTime, art.duration || 0);
      }
    });

    artRef.current = art;
    return () => {
      // 销毁前上报一次最终进度
      try {
        if (art.duration > 0 && art.currentTime > 5) {
          onProgressRef.current?.(art.currentTime, art.duration);
        }
        art.destroy(false);
      } catch {
        /* 忽略 */
      }
      artRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url]);

  return <div className="player-box" ref={boxRef} />;
}
