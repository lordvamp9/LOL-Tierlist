import React, { useState, useRef, useEffect } from 'react';
import { ChevronDown } from 'lucide-react';

export default function ClassicDropdown({
  label,
  options,
  value,
  onChange,
  renderOption,
  renderSelected,
  width = 140,
}) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef(null);

  // Close when clicking outside
  useEffect(() => {
    function handleClickOutside(event) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const selectedOption = options.find((opt) => (opt.id || opt) === value) || options[0];

  return (
    <div className="classic-dropdown-wrapper" ref={dropdownRef} style={{ width }}>
      {label && <span className="dropdown-label">{label}</span>}
      <button
        type="button"
        className={`classic-dropdown-btn ${isOpen ? 'open' : ''}`}
        onClick={() => setIsOpen(!isOpen)}
      >
        <div className="dropdown-selected-content">
          {renderSelected
            ? renderSelected(selectedOption)
            : selectedOption.label || selectedOption}
        </div>
        <ChevronDown size={13} className={`dropdown-arrow ${isOpen ? 'rotated' : ''}`} />
      </button>

      {isOpen && (
        <div className="classic-dropdown-menu">
          {options.map((opt) => {
            const optVal = opt.id || opt;
            const isSelected = optVal === value;
            return (
              <div
                key={optVal}
                className={`classic-dropdown-item ${isSelected ? 'active' : ''}`}
                onClick={() => {
                  onChange(optVal);
                  setIsOpen(false);
                }}
              >
                {renderOption ? renderOption(opt) : opt.label || opt}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
