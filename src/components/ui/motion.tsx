/**
 * Motion 组件 — 弹簧物理动画
 *
 * 功能：
 * - 列表项进入/退出动画
 * - 卡片 hover 弹簧反馈
 * - 面板/弹窗打开动画
 * - 尊重 prefers-reduced-motion 设置
 */

import { useEffect, useState, useRef, type ReactNode } from "react";
import { cn } from "@/lib/utils";

// ─── 动画配置 ──────────────────────────────────────────────

interface SpringConfig {
  stiffness: number;
  damping: number;
  mass: number;
}

const SPRING_PRESETS = {
  // 列表项进入
  listEnter: { stiffness: 300, damping: 30, mass: 1 },
  // 卡片 hover
  cardHover: { stiffness: 400, damping: 25, mass: 1 },
  // 面板打开
  panelOpen: { stiffness: 300, damping: 28, mass: 1 },
  // 弹窗打开
  dialogOpen: { stiffness: 350, damping: 30, mass: 1 },
} as const;

// ─── 弹簧动画计算 ──────────────────────────────────────────────

function springValue(
  progress: number,
  config: SpringConfig
): number {
  // 简化的弹簧物理计算
  const { stiffness, damping, mass } = config;
  const omega = Math.sqrt(stiffness / mass);
  const zeta = damping / (2 * Math.sqrt(stiffness * mass));

  if (zeta < 1) {
    // 欠阻尼
    const omegaD = omega * Math.sqrt(1 - zeta * zeta);
    return 1 - Math.exp(-zeta * omega * progress) *
      (Math.cos(omegaD * progress) + (zeta * omega / omegaD) * Math.sin(omegaD * progress));
  } else if (zeta === 1) {
    // 临界阻尼
    return 1 - (1 + omega * progress) * Math.exp(-omega * progress);
  } else {
    // 过阻尼
    const s1 = -omega * (zeta + Math.sqrt(zeta * zeta - 1));
    const s2 = -omega * (zeta - Math.sqrt(zeta * zeta - 1));
    return 1 - ((s2 * Math.exp(s1 * progress) - s1 * Math.exp(s2 * progress)) / (s2 - s1));
  }
}

// ─── 动画 Hook ──────────────────────────────────────────────

function useReducedMotion(): boolean {
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReduced(query.matches);

    const handler = (e: MediaQueryListEvent) => setReduced(e.matches);
    query.addEventListener("change", handler);
    return () => query.removeEventListener("change", handler);
  }, []);

  return reduced;
}

// ─── Motion 组件 ──────────────────────────────────────────────

interface MotionProps {
  children: ReactNode;
  className?: string;
  /** 动画类型 */
  animation?: "fadeIn" | "slideUp" | "slideDown" | "scaleIn" | "slideInRight";
  /** 弹簧预设 */
  preset?: keyof typeof SPRING_PRESETS;
  /** 动画延迟 (ms) */
  delay?: number;
  /** 是否显示 */
  show?: boolean;
}

export function Motion({
  children,
  className,
  animation = "fadeIn",
  preset = "listEnter",
  delay = 0,
  show = true,
}: MotionProps) {
  const reduced = useReducedMotion();
  const [style, setStyle] = useState<React.CSSProperties>({});
  const frameRef = useRef<number>(0);

  useEffect(() => {
    if (reduced || !show) {
      setStyle({});
      return;
    }

    const config = SPRING_PRESETS[preset];
    const duration = 600; // 动画总时长
    const startTime = performance.now() + delay;

    const animate = (currentTime: number) => {
      const elapsed = currentTime - startTime;
      if (elapsed < 0) {
        frameRef.current = requestAnimationFrame(animate);
        return;
      }

      const progress = Math.min(elapsed / duration, 1);
      const value = springValue(progress * 4, config); // 加速弹簧响应

      let transform = "";
      let opacity = 1;

      switch (animation) {
        case "fadeIn":
          opacity = value;
          break;
        case "slideUp":
          transform = `translateY(${(1 - value) * 20}px)`;
          opacity = value;
          break;
        case "slideDown":
          transform = `translateY(${(1 - value) * -20}px)`;
          opacity = value;
          break;
        case "scaleIn":
          transform = `scale(${0.95 + value * 0.05})`;
          opacity = value;
          break;
        case "slideInRight":
          transform = `translateX(${(1 - value) * 20}px)`;
          opacity = value;
          break;
      }

      setStyle({
        transform,
        opacity,
        transition: "none",
      });

      if (progress < 1) {
        frameRef.current = requestAnimationFrame(animate);
      }
    };

    frameRef.current = requestAnimationFrame(animate);

    return () => {
      if (frameRef.current) {
        cancelAnimationFrame(frameRef.current);
      }
    };
  }, [show, animation, preset, delay, reduced]);

  if (!show) return null;

  return (
    <div className={className} style={style}>
      {children}
    </div>
  );
}

// ─── MotionList 组件 ──────────────────────────────────────────────

interface MotionListProps {
  children: ReactNode;
  className?: string;
  /** 列表项选择器 */
  itemSelector?: string;
}

export function MotionList({ children, className }: MotionListProps) {
  return (
    <div className={className}>
      {children}
    </div>
  );
}

// ─── MotionCard 组件 ──────────────────────────────────────────────

interface MotionCardProps {
  children: ReactNode;
  className?: string;
  /** hover 时是否上浮 */
  hoverable?: boolean;
}

export function MotionCard({
  children,
  className,
  hoverable = true,
}: MotionCardProps) {
  const reduced = useReducedMotion();

  return (
    <div
      className={cn(
        "transition-transform",
        hoverable && !reduced && "hover:-translate-y-0.5 hover:shadow-lg",
        className
      )}
    >
      {children}
    </div>
  );
}

// ─── AnimatePresence 组件 ──────────────────────────────────────────────

interface AnimatePresenceProps {
  children: ReactNode;
  show: boolean;
  animation?: MotionProps["animation"];
  preset?: MotionProps["preset"];
}

export function AnimatePresence({
  children,
  show,
  animation = "scaleIn",
  preset = "dialogOpen",
}: AnimatePresenceProps) {
  return (
    <Motion show={show} animation={animation} preset={preset}>
      {children}
    </Motion>
  );
}
