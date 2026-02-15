"use client";

import { motion, AnimatePresence } from "framer-motion";
import { useState } from "react";
import Link from "next/link";
import {
  Atom,
  Link as LinkIcon,
  Eye,
  ArrowBigLeft,
  ChevronRight,
  ShieldAlert,
} from "lucide-react";

export default function HowToPlay() {
  return (
    <div className="game-container htp-container">
      {/* Header */}
      <header className="htp-header">
        <Link href="/" className="htp-back-link">
          <ArrowBigLeft size={20} className="mr-1" />
          Back to Base
        </Link>
        <h1 className="htp-title">Cadet Training Module</h1>
        <p className="htp-subtitle">
          Master the laws of quantum mechanics to survive the minefield.
        </p>
      </header>

      <div className="htp-content">
        <SuperpositionSection />
        <EntanglementSection />
        <ObserverSection />
      </div>

      <footer className="htp-cta">
        <Link href="/" className="btn-cta">
          Ready for Deployment
          <ChevronRight size={20} className="ml-2" />
        </Link>
      </footer>
    </div>
  );
}

function SuperpositionSection() {
  const [isObserved, setIsObserved] = useState(false);
  const [value, setValue] = useState<0 | 1>(0);

  const toggleObservation = () => {
    if (!isObserved) {
      // Simulate random collapse
      setValue(Math.random() > 0.5 ? 1 : 0);
    }
    setIsObserved(!isObserved);
  };

  return (
    <section className="glass htp-section">
      <div className="htp-section-header">
        <div className="section-icon-box">
          <Atom size={24} />
        </div>
        <h2>Superposition</h2>
      </div>

      <div className="htp-grid">
        <div className="htp-text">
          <p>
            Unlike classical mines, a Quantum Mine exists in a state of{" "}
            <strong>Superposition</strong>. It is both safe (|0⟩) and a mine
            (|1⟩) at the same time until observed.
          </p>
          <div className="htp-tip">
            <strong>Cadet Tip:</strong> The numbers on the grid represent
            probability amplitudes, not certainty. Use the Quantum Inspector
            tool to see the exact odds.
          </div>
        </div>

        <div className="htp-demo-area">
          <div
            className="qubit-demo"
            onClick={toggleObservation}
            style={{
              cursor: "pointer",
              marginBottom: "1.5rem",
              position: "relative",
              width: "100px",
              height: "100px",
            }}
          >
            <AnimatePresence mode="wait">
              {!isObserved ? (
                <motion.div
                  key="superposition"
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                  style={{
                    position: "absolute",
                    inset: 0,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  <motion.div
                    animate={{ rotate: 360 }}
                    transition={{
                      duration: 3,
                      repeat: Infinity,
                      ease: "linear",
                    }}
                    style={{
                      width: "80px",
                      height: "80px",
                      borderRadius: "50%",
                      border: "3px dashed rgba(125, 211, 252, 0.5)",
                      position: "absolute",
                    }}
                  />
                  <motion.div
                    animate={{ rotate: -360 }}
                    transition={{
                      duration: 4,
                      repeat: Infinity,
                      ease: "linear",
                    }}
                    style={{
                      width: "50px",
                      height: "50px",
                      borderRadius: "50%",
                      border: "3px dashed rgba(167, 139, 250, 0.5)",
                      position: "absolute",
                    }}
                  />
                  <span
                    style={{
                      fontSize: "1.5rem",
                      fontFamily: "monospace",
                      color: "var(--accent)",
                    }}
                  >
                    |ψ⟩
                  </span>
                </motion.div>
              ) : (
                <motion.div
                  key="collapsed"
                  initial={{ opacity: 0, scale: 1.2 }}
                  animate={{ opacity: 1, scale: 1 }}
                  style={{
                    width: "80px",
                    height: "80px",
                    borderRadius: "12px",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: "2.5rem",
                    fontWeight: "bold",
                    background:
                      value === 1
                        ? "rgba(248, 113, 113, 0.2)"
                        : "rgba(74, 222, 128, 0.2)",
                    color: value === 1 ? "var(--danger)" : "var(--success)",
                    border: `2px solid ${value === 1 ? "var(--danger)" : "var(--success)"}`,
                  }}
                >
                  {value === 1 ? "☢" : "0"}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
          <button
            onClick={toggleObservation}
            className="btn-secondary"
            style={{
              width: "auto",
              padding: "0.5rem 1rem",
              fontSize: "0.85rem",
            }}
          >
            {isObserved ? "Reset Qubit" : "Observe State"}
          </button>
        </div>
      </div>
    </section>
  );
}

function EntanglementSection() {
  const [activeId, setActiveId] = useState<number | null>(null);

  return (
    <section className="glass htp-section">
      <div className="htp-section-header">
        <div
          className="section-icon-box"
          style={{ color: "#a78bfa", background: "rgba(167, 139, 250, 0.15)" }}
        >
          <LinkIcon size={24} />
        </div>
        <h2>Quantum Entanglement</h2>
      </div>

      <div className="htp-grid">
        <div className="htp-text">
          <p style={{ marginBottom: "1rem" }}>
            Particles can become <strong>Entangled</strong>. Changing the state
            of one instantaneously affects its partner, regardless of distance.
          </p>
          <ul style={{ paddingLeft: "1.2rem", listStyleType: "disc" }}>
            <li>
              If Particle A is observed as Safe, Particle B becomes more likely
              to be a Mine.
            </li>
            <li>Use this correlation to solve impossible setups.</li>
          </ul>
        </div>

        <div className="htp-demo-area" style={{ minHeight: "180px" }}>
          <div style={{ position: "relative", display: "flex", gap: "3rem" }}>
            {/* Connection Line */}
            <svg
              style={{
                position: "absolute",
                top: "50%",
                left: 0,
                width: "100%",
                height: "1px",
                overflow: "visible",
                pointerEvents: "none",
              }}
            >
              <motion.path
                d="M 25 -20 Q 75 20, 125 -20" // crude approx
                fill="none"
                stroke="url(#entanglement-gradient)"
                strokeWidth="2"
                strokeDasharray="4 4"
                animate={{ strokeDashoffset: [0, -8] }}
                transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
              />
              <defs>
                <linearGradient
                  id="entanglement-gradient"
                  x1="0%"
                  y1="0%"
                  x2="100%"
                  y2="0%"
                >
                  <stop offset="0%" stopColor="#a855f7" />
                  <stop offset="100%" stopColor="#3b82f6" />
                </linearGradient>
              </defs>
            </svg>

            {[0, 1].map((id) => (
              <motion.button
                key={id}
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.95 }}
                onClick={() => setActiveId(id)}
                style={{
                  position: "relative",
                  zIndex: 10,
                  width: "50px",
                  height: "50px",
                  borderRadius: "10px",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: "1.2rem",
                  fontWeight: "bold",
                  border: "2px solid",
                  cursor: "pointer",
                  transition: "all 0.2s",
                  background:
                    activeId === id
                      ? "#a78bfa"
                      : activeId !== null
                        ? "#38bdf8"
                        : "rgba(30, 41, 59, 1)",
                  borderColor:
                    activeId === id
                      ? "#c4b5fd"
                      : activeId !== null
                        ? "#7dd3fc"
                        : "rgba(71, 85, 105, 1)",
                  color: activeId === null ? "#94a3b8" : "#fff",
                  boxShadow:
                    activeId === id
                      ? "0 0 15px rgba(167, 139, 250, 0.5)"
                      : "none",
                  transform:
                    activeId !== null && activeId !== id
                      ? "scale(0.9)"
                      : "none",
                  opacity: activeId !== null && activeId !== id ? 0.8 : 1,
                }}
              >
                {activeId === id ? "A" : activeId !== null ? "B" : "?"}
              </motion.button>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function ObserverSection() {
  const [isHovering, setIsHovering] = useState(false);

  return (
    <section className="glass htp-section">
      <div className="htp-section-header">
        <div
          className="section-icon-box"
          style={{ color: "#4ade80", background: "rgba(74, 222, 128, 0.15)" }}
        >
          <Eye size={24} />
        </div>
        <h2>The Observer Effect</h2>
      </div>

      <div className="htp-grid">
        <div className="htp-text">
          <p style={{ marginBottom: "1rem" }}>
            Observation is not passive. Simply <strong>measuring</strong> a
            quantum system perturbs its state. This introduces noise and drift
            into probability calculations.
          </p>
          <div
            className="htp-tip"
            style={{
              display: "flex",
              gap: "0.75rem",
              borderColor: "rgba(74, 222, 128, 0.3)",
              color: "#e6f0ff",
            }}
          >
            <ShieldAlert
              size={20}
              className="shrink-0"
              style={{ color: "var(--success)" }}
            />
            <div>
              <strong>Warning:</strong> The more you inspect a cell without
              resolving it, the less accurate your probability readouts become.
            </div>
          </div>
        </div>

        <div className="htp-demo-area">
          <motion.div
            onHoverStart={() => setIsHovering(true)}
            onHoverEnd={() => setIsHovering(false)}
            style={{ position: "relative", cursor: "crosshair" }}
          >
            <div
              style={{
                width: "120px",
                height: "80px",
                background: "#0f172a",
                borderRadius: "8px",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                border: "1px solid #334155",
                overflow: "hidden",
                position: "relative",
              }}
            >
              <motion.div
                animate={
                  isHovering
                    ? {
                        x: [0, -5, 5, -3, 3, 0],
                        y: [0, 3, -3, 2, -2, 0],
                        filter: ["blur(0px)", "blur(2px)", "blur(0px)"],
                      }
                    : {}
                }
                transition={{ duration: 0.5, repeat: Infinity }}
                style={{
                  fontSize: "1.5rem",
                  fontFamily: "monospace",
                  color: "var(--success)",
                }}
              >
                {isHovering ? "??%" : "50%"}
              </motion.div>
            </div>

            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: isHovering ? 1 : 0, y: isHovering ? 0 : 10 }}
              style={{
                position: "absolute",
                top: "-30px",
                left: "50%",
                translateX: "-50%",
                background: "var(--success)",
                color: "#0f172a",
                fontSize: "0.7rem",
                fontWeight: "bold",
                padding: "2px 6px",
                borderRadius: "4px",
                whiteSpace: "nowrap",
              }}
            >
              System Perturbed!
            </motion.div>
          </motion.div>

          <p
            style={{
              marginTop: "1rem",
              fontSize: "0.75rem",
              color: "var(--text-muted)",
            }}
          >
            {isHovering
              ? "Applying Measurement Operator..."
              : "Hover to Measure"}
          </p>
        </div>
      </div>
    </section>
  );
}
