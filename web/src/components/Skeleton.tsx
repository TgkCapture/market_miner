export function Skeleton({ height = 16, width = "100%", radius = 8 }: {
  height?: number | string;
  width?: number | string;
  radius?: number;
}) {
  return <span className="skeleton" style={{ height, width, borderRadius: radius }} aria-hidden="true" />;
}

export function OverviewSkeleton() {
  return (
    <div aria-busy="true" aria-label="Loading market data">
      <div className="stats">
        {[0, 1, 2, 3].map((i) => (
          <div key={i} className="card stat"><Skeleton width="40%" height={12} /><Skeleton width="70%" height={28} /></div>
        ))}
      </div>
      <div className="movers">
        {[0, 1, 2].map((i) => (
          <div key={i} className="card movers-card">
            {[0, 1, 2, 3, 4].map((j) => <Skeleton key={j} height={20} />)}
          </div>
        ))}
      </div>
      <div className="card table-card">
        {Array.from({ length: 8 }, (_, i) => <Skeleton key={i} height={22} />)}
      </div>
    </div>
  );
}
