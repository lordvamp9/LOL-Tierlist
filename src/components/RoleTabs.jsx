import React from 'react';
import { Shield, Compass, Swords, Crosshair, Heart } from 'lucide-react';

const ROLES = [
  { id: 'TOP', label: 'TOP', icon: Shield },
  { id: 'JUNGLE', label: 'JUNGLE', icon: Compass },
  { id: 'MID', label: 'MID', icon: Swords },
  { id: 'CARRY', label: 'CARRY', icon: Crosshair },
  { id: 'SUPPORT', label: 'SUPPORT', icon: Heart },
];

export default function RoleTabs({ activeRole, onSelectRole }) {
  return (
    <div className="role-tabs-container">
      {ROLES.map(({ id, label, icon: Icon }) => {
        const isActive = activeRole === id;
        return (
          <button
            key={id}
            className={`role-tab-btn ${isActive ? 'active' : ''}`}
            onClick={() => onSelectRole(id)}
          >
            <Icon size={14} className="role-tab-icon" />
            <span>{label}</span>
          </button>
        );
      })}
    </div>
  );
}
