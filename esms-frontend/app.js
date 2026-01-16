const WS_URL='ws://localhost:8080/ws';const MAX_POINTS=60;
const data={temp:[],humidity:[],sound:[],hr:[]};
let ws;const charts={};
function connect(){
  const status=document.getElementById('connectionStatus');
  status.textContent='Connecting...';status.className='connection-status';
  ws=new WebSocket(WS_URL);
  ws.onopen=()=>{status.textContent='Connected';status.className='connection-status connected'};
  ws.onclose=()=>{status.textContent='Disconnected';status.className='connection-status error';setTimeout(connect,3000)};
  ws.onerror=()=>{status.textContent='Error';status.className='connection-status error'};
  ws.onmessage=(e)=>{
    try{const msg=JSON.parse(e.data);if(msg.type==='SensorUpdate')updateData(msg.data)}catch(err){console.error(err)}
  };
}
function updateData(r){
  const t=new Date(r.timestamp);
  [{arr:data.temp,val:r.temperature,el:'tempValue',chart:'tempChart'},
   {arr:data.humidity,val:r.humidity,el:'humidityValue',chart:'humidityChart'},
   {arr:data.sound,val:r.sound,el:'soundValue',chart:'soundChart'},
   {arr:data.hr,val:r.heart_rate,el:'hrValue',chart:'hrChart'}
  ].forEach(({arr,val,el,chart})=>{
    arr.push({time:t,value:val});if(arr.length>MAX_POINTS)arr.shift();
    document.getElementById(el).textContent=val.toFixed(1);
    updateChart(chart,arr);
  });
  updateIndicators(r);
}
function updateChart(id,arr){
  const c=document.getElementById(id);if(!arr.length)return;
  const w=c.clientWidth,h=c.clientHeight,m={t:10,r:10,b:20,l:40};
  const x=d3.scaleTime().domain(d3.extent(arr,d=>d.time)).range([m.l,w-m.r]);
  const y=d3.scaleLinear().domain([d3.min(arr,d=>d.value)*0.95,d3.max(arr,d=>d.value)*1.05]).range([h-m.b,m.t]);
  const line=d3.line().x(d=>x(d.time)).y(d=>y(d.value)).curve(d3.curveMonotoneX);
  const area=d3.area().x(d=>x(d.time)).y0(h-m.b).y1(d=>y(d.value)).curve(d3.curveMonotoneX);
  d3.select(c).selectAll('*').remove();
  const svg=d3.select(c).append('svg').attr('viewBox',`0 0 ${w} ${h}`);
  const grad=svg.append('defs').append('linearGradient').attr('id',id+'Grad').attr('x1','0%').attr('y1','0%').attr('x2','0%').attr('y2','100%');
  grad.append('stop').attr('offset','0%').attr('stop-color','#00d4aa').attr('stop-opacity',0.3);
  grad.append('stop').attr('offset','100%').attr('stop-color','#00d4aa').attr('stop-opacity',0);
  svg.append('path').datum(arr).attr('fill',`url(#${id}Grad)`).attr('d',area);
  svg.append('path').datum(arr).attr('fill','none').attr('stroke','#00d4aa').attr('stroke-width',2).attr('d',line);
}
function updateIndicators(r){
  const el=document.getElementById('stressIndicators');el.innerHTML='';
  const ind=[];
  if(r.temperature>28)ind.push({t:'High Temperature',h:true});
  if(r.temperature<18)ind.push({t:'Low Temperature',h:false});
  if(r.humidity>70)ind.push({t:'High Humidity',h:false});
  if(r.humidity<30)ind.push({t:'Low Humidity',h:false});
  if(r.sound>500)ind.push({t:'High Noise',h:true});
  if(r.heart_rate>100)ind.push({t:'Elevated HR',h:true});
  if(r.heart_rate<50)ind.push({t:'Low HR',h:false});
  ind.forEach(i=>{const d=document.createElement('div');d.className='stress-indicator'+(i.h?' high':'');d.textContent=i.t;el.appendChild(d)});
  if(!ind.length){const d=document.createElement('div');d.className='stress-indicator';d.style.background='rgba(0,212,170,0.15)';d.style.color='#00d4aa';d.textContent='All metrics normal';el.appendChild(d)}
}
connect();