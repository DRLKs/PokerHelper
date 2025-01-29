import java.awt.BorderLayout;
import java.awt.Color;
import java.awt.GridLayout;
import java.awt.event.ActionListener;

import javax.swing.JButton;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JProgressBar;
import javax.swing.JTextField;

public class Panel extends JPanel {
	
	/**
	 * 
	 */
	private static final long serialVersionUID = 7053570639591241429L;

	// ETIQUETAS
	/* Estas 2 primeras quizás se usan */
	//private final JLabel etiquetaInstruccionesPalos = new JLabel("Trebol = T\nPica = P\nDiamante = D\nCorazon = C");
	//private final JLabel etiquetaInstruccionesCartas = new JLabel("J = 11 Q = 12 K = 13 AS = 1 ");
	
	private JLabel etiquetaDecion = new JLabel("");	// Resultado del modelo

	private final JLabel etiquetaCartasMano = new JLabel("CARTAS MANO");
	private final JLabel etiquetaCartasMesa = new JLabel("CARTAS MESA");
	private final JLabel etiquetaJugadoresActivos = new JLabel("JUGADORES ACTIVOS");
	
	// BOTONES
	private JButton nuevaPartida = new JButton("NUEVA PARTIDA");

	// RELLENAR CARTAS EN JUEGO
	private JTextField entradaCartasMano1 = new JTextField(3);
	private JTextField entradaCartasMano2 = new JTextField(3);
	
	private JTextField entradaCartasMesa1 = new JTextField(3);
	private JTextField entradaCartasMesa2 = new JTextField(3);
	private JTextField entradaCartasMesa3 = new JTextField(3);
	private JTextField entradaCartasMesa4 = new JTextField(3);
	private JTextField entradaCartasMesa5 = new JTextField(3);
	
	private JTextField entradaJugadoresActivos = new JTextField(0);

	// PROGRESS PANEL
	private JProgressBar porcentajePareja = new JProgressBar(0, 100);
	private JProgressBar porcentajeTrio = new JProgressBar(0, 100);
	private JProgressBar porcentajeEscalera = new JProgressBar(0, 100);
	private JProgressBar porcentajeColor = new JProgressBar(0, 100);
	private JProgressBar porcentajeFull = new JProgressBar(0, 100);
	private JProgressBar porcentajePoker = new JProgressBar(0, 100);
	private JProgressBar porcentajeEscaleraColor = new JProgressBar(0, 100);
	private JProgressBar porcentajeEscaleraReal = new JProgressBar(0, 100);

	
	
	public void controlador(ActionListener ctr) {
		
		nuevaPartida.setActionCommand("NUEVAPARTIDA");
		
		entradaJugadoresActivos.setActionCommand("NUMJUGADORES");
		entradaCartasMesa1.setActionCommand("CARTAMESA1");
		entradaCartasMesa2.setActionCommand("CARTAMESA2");
		entradaCartasMesa3.setActionCommand("CARTAMESA3");
		entradaCartasMesa4.setActionCommand("CARTAMESA4");
		entradaCartasMesa5.setActionCommand("CARTAMESA5");
		entradaCartasMano1.setActionCommand("CARTAMANO1");
		entradaCartasMano2.setActionCommand("CARTAMANO2");

		
		nuevaPartida.addActionListener(ctr);

		entradaJugadoresActivos.addActionListener(ctr);
		entradaCartasMesa1.addActionListener(ctr);
		entradaCartasMesa2.addActionListener(ctr);
		entradaCartasMesa3.addActionListener(ctr);
		entradaCartasMesa4.addActionListener(ctr);
		entradaCartasMesa5.addActionListener(ctr);
		entradaCartasMano1.addActionListener(ctr);
		entradaCartasMano2.addActionListener(ctr);
	}
	
	public Panel() {
				
		this.setLayout(new BorderLayout());
		
		// NORTE DEL PANEL
		
		JPanel norte = new JPanel();
		JPanel norteIzq = new JPanel();
		norteIzq.setLayout( new GridLayout(2,1) );
		norteIzq.add(etiquetaJugadoresActivos);
		norteIzq.add(entradaJugadoresActivos);
		norte.add(norteIzq);
		norte.add(nuevaPartida);
		this.add(BorderLayout.NORTH, norte);
		
		// CENTRO DEL PANEL
		
		JPanel centro = new JPanel();
		centro.setLayout(new GridLayout(3, 1));

		JPanel centroArriba = new JPanel();
		centroArriba.add(etiquetaCartasMesa);
		centro.add(BorderLayout.NORTH, centroArriba);
		
		JPanel centroCentro = new JPanel();
		centroCentro.setLayout(new GridLayout(1, 3));
		centroCentro.add(entradaCartasMesa1);
		centroCentro.add(entradaCartasMesa2);
		centroCentro.add(entradaCartasMesa3);
		centro.add(BorderLayout.CENTER, centroCentro);
		
		JPanel centroAbajo = new JPanel();
		centroAbajo.setLayout(new GridLayout(1, 2));
		centroAbajo.add(entradaCartasMesa4);
		centroAbajo.add(entradaCartasMesa5);
		centro.add(BorderLayout.SOUTH, centroAbajo);
		
		this.add(BorderLayout.CENTER, centro);
		
		// SUR DEL PANEL
		
		JPanel sur = new JPanel();
		sur.setLayout(new GridLayout(4, 1));
		
		JPanel surArriba = new JPanel();
		surArriba.add(etiquetaCartasMano);
		sur.add(BorderLayout.NORTH, surArriba);
		
		JPanel surCentro = new JPanel();
		surCentro.setLayout( new GridLayout(1,2) );
		surCentro.add(entradaCartasMano1);
		surCentro.add(entradaCartasMano2);
		sur.add(BorderLayout.CENTER, surCentro);
		
		JPanel surAbajo = new JPanel();
		// Porcentajes
		surAbajo.add(porcentajePareja);
		surAbajo.add(porcentajeTrio);
		surAbajo.add(porcentajeEscalera);
		surAbajo.add(porcentajeColor);
		
		JPanel surAbajoAbajo = new JPanel();

		surAbajoAbajo.add(porcentajeFull);
		surAbajoAbajo.add(porcentajePoker);
		surAbajoAbajo.add(porcentajeEscaleraColor);
		surAbajoAbajo.add(porcentajeEscaleraReal);
		
		surAbajoAbajo.add(etiquetaDecion);
		
		sur.add(BorderLayout.CENTER, surAbajo);
		sur.add(BorderLayout.CENTER, surAbajoAbajo);

		this.add(BorderLayout.SOUTH, sur);
		
		// CONFIGURACION DE ETIQUETAS
		entradaCartasMano1.setHorizontalAlignment(JTextField.CENTER);
		entradaCartasMano2.setHorizontalAlignment(JTextField.CENTER);
		entradaCartasMesa3.setHorizontalAlignment(JTextField.CENTER);
		entradaCartasMesa1.setHorizontalAlignment(JTextField.CENTER);
		entradaCartasMesa2.setHorizontalAlignment(JTextField.CENTER);
		entradaCartasMesa3.setHorizontalAlignment(JTextField.CENTER);
		entradaCartasMesa4.setHorizontalAlignment(JTextField.CENTER);
		entradaCartasMesa5.setHorizontalAlignment(JTextField.CENTER);
		entradaJugadoresActivos.setHorizontalAlignment(JTextField.CENTER);
		
		// Inicialización Porcentajes
		porcentajePareja.setStringPainted(true);
		setProbabilidad(porcentajePareja, 100);
		
		porcentajeTrio.setStringPainted(true);
		setProbabilidad(porcentajeTrio, 100);
		
		porcentajeEscalera.setStringPainted(true);
		setProbabilidad(porcentajeEscalera, 100);
		
		porcentajeColor.setStringPainted(true);
		setProbabilidad(porcentajeColor, 100);
		
		porcentajeFull.setStringPainted(true);
		setProbabilidad(porcentajeFull, 100);
		
		porcentajePoker.setStringPainted(true);
		setProbabilidad(porcentajePoker, 100);
		
		porcentajeEscaleraColor.setStringPainted(true);
		setProbabilidad(porcentajeEscaleraColor, 100);
		
		porcentajeEscaleraReal.setStringPainted(true);
		setProbabilidad(porcentajeEscaleraReal, 100);

	}
	
	public void limpiarAreasCartas() {
		entradaJugadoresActivos.setText("");
		entradaCartasMano1.setText("");
		entradaCartasMano2.setText("");
		entradaCartasMesa1.setText("");
		entradaCartasMesa2.setText("");
		entradaCartasMesa3.setText("");
		entradaCartasMesa4.setText("");
		entradaCartasMesa5.setText("");
	}
	
	public int numJugadoresActuales() throws NumberFormatException, PokerException {
		Integer numJugadores = Integer.parseInt( entradaJugadoresActivos.getText().trim() );
		if( numJugadores > 8 ) {
			throw new PokerException("MUCHOS JUGADORES");
		}else if( numJugadores < 0 ) {
			throw new PokerException("JUGADORES NEGATIVOS");
		}
		return numJugadores;
	}
	
	public String getEntradas( int codigo ) {
		
		String salida;
		if( codigo == 1 ) {
			salida = entradaCartasMano1.getText().trim();	
		}else if( codigo == 2 ) {
			salida = entradaCartasMano2.getText().trim();	
		}else if( codigo == 3 ) {
			salida = entradaCartasMesa1.getText().trim();
		}else if( codigo == 4 ) {
			salida = entradaCartasMesa2.getText().trim();
		}else if( codigo == 5 ) {
			salida = entradaCartasMesa3.getText().trim();
		}else if( codigo == 6 ) {
			salida = entradaCartasMesa4.getText().trim();
		}else if( codigo == 7 ) {
			salida = entradaCartasMesa5.getText().trim();
		}else {
			salida = null;
		}
		return salida;
	}
	
	void setProbabilidadPareja(int p) {
		setProbabilidad(porcentajePareja,p);
	}
	
	void setProbabilidadTrio(int p) {
		setProbabilidad(porcentajeTrio,p);
	}
	
	void setProbabilidadEscalera(int p) {
		setProbabilidad(porcentajeEscalera,p);
	}
	
	void setProbabilidadColor(int p) {
		setProbabilidad(porcentajeColor,p);
	}
	
	void setProbabilidadFull(int p) {
		setProbabilidad(porcentajeFull,p);
	}
	
	void setProbabilidadPoker(int p) {
		setProbabilidad(porcentajePoker,p);
	}
	
	void setProbabilidadEscaleraColor(int p) {
		setProbabilidad(porcentajeEscaleraColor,p);
	}
	
	void setProbabilidadEscaleraReal(int p) {
		setProbabilidad(porcentajeEscaleraReal,p);
	}
	
	
	private void setProbabilidad( JProgressBar jpb,  int p) {
		if( p > 65 ) {
			jpb.setForeground( new Color(0,160,0) );
		}else if( p > 40 ) {
			jpb.setForeground( new Color(255,165,0) );
		}else {
			jpb.setForeground( new Color(160,0,0) );
		}
		jpb.setValue(p);
	}
	
	public void setDecision( String msg ) {
		etiquetaDecion.setText(msg);
	}
	
	public void clearDecision() {
		etiquetaDecion.setText("");
	}
	
	/** 
	 * Por implementar, en un futuro
	 * La idea es que salga un despegable, un JFrame con todas las cartas seleccionables
	 */
	@SuppressWarnings("unused")
	private void interfazSeleccionarCartas() {
		
	}
}